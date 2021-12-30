// Copyright © Spelldawn 2021-present

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at

//    https://www.apache.org/licenses/LICENSE-2.0

// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Core module responsible for turning game updates into GRPC protobuf
//! responses.

use data::card_definition::CardDefinition;
use data::card_state::{CardPosition, CardPositionKind, CardState};
use data::game::GameState;
use data::primitives::{CardId, CardType, ItemLocation, RoomId, RoomLocation, Side, Sprite};
use data::updates::GameUpdate;
use protos::spelldawn::card_targeting::Targeting;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{
    game_object_identifier, ActionTrackerView, ArenaView, CanPlayAlgorithm, CardCreationAnimation,
    CardIcon, CardIcons, CardIdentifier, CardTargeting, CardTitle, CardView, CardViewCost,
    ClientItemLocation, ClientRoomLocation, CreateOrUpdateCardCommand, DelayCommand, GameCommand,
    GameIdentifier, GameObjectIdentifier, GameView, IdentityAction, ManaView,
    MoveGameObjectsCommand, ObjectPosition, ObjectPositionDeck, ObjectPositionDiscardPile,
    ObjectPositionHand, ObjectPositionIdentity, ObjectPositionItem, ObjectPositionRoom,
    ObjectPositionStaging, PickRoom, PlayerInfo, PlayerName, PlayerSide, PlayerView,
    RevealedCardView, RoomIdentifier, ScoreView, SpendCostAlgorithm, SpriteAddress, TimeValue,
    UpdateGameViewCommand,
};
use rules::queries;

use crate::assets::CardIconType;
use crate::{assets, rules_text};

/// Builds a series of [GameCommand]s to fully represent the current state of
/// this game in the client, for use e.g. in response to a reconnect request.
pub fn full_sync(game: &GameState, user_side: Side) -> Vec<GameCommand> {
    let mut result =
        vec![GameCommand { command: Some(game_view(game, user_side, GameUpdateType::Full)) }];
    result.extend(
        game.all_cards()
            .filter(|c| c.position.kind() != CardPositionKind::DeckUnknown)
            .map(|c| {
                create_or_update_card(
                    game,
                    c,
                    user_side,
                    CardCreationStrategy::SnapToCurrentPosition,
                )
            })
            .map(|c| GameCommand { command: Some(c) }),
    );
    result
}

/// Builds a series of [GameCommand]s to represent the updates present in
/// [GameState::updates].
pub fn render_updates(game: &GameState, user_side: Side) -> Vec<GameCommand> {
    game.updates.update_list.as_ref().map_or_else(Vec::new, |updates| {
        updates
            .iter()
            .flat_map(|update| adapt_update(game, user_side, *update))
            .map(|c| GameCommand { command: Some(c) })
            .collect()
    })
}

/// Converts a [GameUpdate] into a [Command] list describing the required client
/// changes.
pub fn adapt_update(game: &GameState, user_side: Side, update: GameUpdate) -> Vec<Command> {
    match update {
        GameUpdate::UpdateGameState => vec![game_view(game, user_side, GameUpdateType::State)],
        GameUpdate::UpdateCard(card_id) => vec![create_or_update_card(
            game,
            game.card(card_id),
            user_side,
            CardCreationStrategy::SnapToCurrentPosition,
        )],
        GameUpdate::DrawCard(card_id) => draw_card(game, game.card(card_id), user_side),
        GameUpdate::MoveCard(card_id) => {
            filtered(vec![move_card(game, game.card(card_id), user_side)])
        }
        GameUpdate::RevealCard(card_id) => reveal_card(game, game.card(card_id), user_side),
        _ => todo!(),
    }
}

/// Possible behavior when updating the state of a game
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
enum GameUpdateType {
    /// Sync all game state, including arena info and player info
    Full,
    /// Sync only normally mutable game state, such as player mana and current
    /// priority
    State,
}

/// Builds a command to update the client's current [GameView]
fn game_view(game: &GameState, user_side: Side, update_type: GameUpdateType) -> Command {
    Command::UpdateGameView(UpdateGameViewCommand {
        game: Some(GameView {
            game_id: Some(GameIdentifier { value: game.id.value }),
            user: Some(player_view(game, user_side, update_type)),
            opponent: Some(player_view(game, user_side.opponent(), update_type)),
            arena: if update_type == GameUpdateType::State {
                None
            } else {
                Some(ArenaView {
                    rooms_at_bottom: Some(user_side == Side::Overlord),
                    identity_action: match user_side {
                        Side::Overlord => IdentityAction::LevelUpRoom.into(),
                        Side::Champion => IdentityAction::InitiateRaid.into(),
                    },
                })
            },
            current_priority: current_priority(game, user_side).into(),
        }),
    })
}

/// Builds commands to represent a card being drawn
fn draw_card(game: &GameState, card: &CardState, user_side: Side) -> Vec<Command> {
    filtered(vec![
        Some(create_or_update_card(
            game,
            card,
            user_side,
            if card.side == user_side {
                CardCreationStrategy::DrawUserCard
            } else {
                CardCreationStrategy::CreateAtPosition(ObjectPosition {
                    sorting_key: u32::MAX,
                    position: Some(Position::Deck(ObjectPositionDeck {
                        owner: PlayerName::Opponent.into(),
                    })),
                })
            },
        )),
        move_card(game, card, user_side),
    ])
}

/// Creates a move card command to move a card to its current location. Returns
/// None if the destination would not be a valid game position, e.g. if it is
/// [CardPosition::DeckUnknown].
fn move_card(game: &GameState, card: &CardState, user_side: Side) -> Option<Command> {
    adapt_position(card, game.card(card.id).position, user_side).map(|position| {
        Command::MoveGameObjects(MoveGameObjectsCommand {
            ids: vec![adapt_game_object_id(card.id)],
            position: Some(position),
            disable_animation: false,
        })
    })
}

/// Possible behavior when creating a card
#[derive(Debug, PartialEq, Clone)]
enum CardCreationStrategy {
    /// Animate the card moving from the user's deck to the staging area.
    DrawUserCard,
    /// Jump the newly-created card to its current game position. If the current
    /// position is invalid (e.g. in the user's deck), no initial position
    /// will be specified for the card.
    SnapToCurrentPosition,
    /// Create the card at a specific game object position.
    CreateAtPosition(ObjectPosition),
}

/// Creates a create/update card command.
fn create_or_update_card(
    game: &GameState,
    card: &CardState,
    user_side: Side,
    creation_strategy: CardCreationStrategy,
) -> Command {
    let definition = rules::get(card.name);
    let revealed = card.is_revealed_to(user_side);
    let create_animation = if creation_strategy == CardCreationStrategy::DrawUserCard {
        CardCreationAnimation::DrawCard.into()
    } else {
        CardCreationAnimation::Unspecified.into()
    };
    let position = match creation_strategy {
        CardCreationStrategy::DrawUserCard => None,
        CardCreationStrategy::SnapToCurrentPosition => {
            adapt_position(card, game.card(card.id).position, user_side)
        }
        CardCreationStrategy::CreateAtPosition(p) => Some(p),
    };

    Command::CreateOrUpdateCard(CreateOrUpdateCardCommand {
        card: Some(CardView {
            card_id: Some(adapt_card_id(card.id)),
            card_icons: Some(card_icons(game, card, definition, revealed)),
            arena_frame: Some(assets::arena_frame(
                definition.side,
                definition.card_type,
                definition.config.faction,
            )),
            owning_player: to_player_name(definition.side, user_side).into(),
            revealed_card: revealed.then(|| revealed_card(game, card, definition, user_side)),
        }),
        create_position: position,
        create_animation,
        disable_flip_animation: false,
    })
}

/// Commands to reveal the indicated card to a player
fn reveal_card(game: &GameState, card: &CardState, user_side: Side) -> Vec<Command> {
    let mut result = vec![create_or_update_card(
        game,
        card,
        user_side,
        CardCreationStrategy::SnapToCurrentPosition,
    )];

    if user_side != card.side {
        result.push(Command::MoveGameObjects(MoveGameObjectsCommand {
            ids: vec![adapt_game_object_id(card.id)],
            position: Some(ObjectPosition {
                sorting_key: 0,
                position: Some(Position::Staging(ObjectPositionStaging {})),
            }),
            disable_animation: false,
        }));
        result.push(delay(1500));
    }
    result
}

/// Builds a [PlayerView] for a given player
fn player_view(game: &GameState, side: Side, update_type: GameUpdateType) -> PlayerView {
    let identity = game.identity(side);
    let data = game.player(side);
    PlayerView {
        player_info: if update_type == GameUpdateType::State {
            None
        } else {
            Some(PlayerInfo {
                name: identity.name.displayed_name(),
                portrait: Some(sprite(&rules::get(identity.name).image)),
                portrait_frame: Some(assets::identity_card_frame(side)),
                card_back: Some(assets::card_back(rules::get(identity.name).school)),
            })
        },
        score: Some(ScoreView { score: data.score }),
        mana: Some(ManaView { amount: data.mana }),
        action_tracker: Some(ActionTrackerView { available_action_count: data.actions }),
    }
}

/// Returns the [PlayerName] that currently has priority (is next to act) in
/// this game
fn current_priority(game: &GameState, user_side: Side) -> PlayerName {
    to_player_name(
        match game.data.raid {
            Some(raid) => raid.priority,
            None => game.data.turn,
        },
        user_side,
    )
}

/// Build icons struct for this card
fn card_icons(
    game: &GameState,
    card: &CardState,
    definition: &CardDefinition,
    revealed: bool,
) -> CardIcons {
    if revealed {
        CardIcons {
            top_left_icon: queries::mana_cost(game, card.id).map(|mana| CardIcon {
                background: Some(assets::card_icon(CardIconType::Mana)),
                text: mana.to_string(),
                background_scale: 1.0,
            }),
            bottom_left_icon: definition.config.stats.shield.map(|_| CardIcon {
                background: Some(assets::card_icon(CardIconType::Shield)),
                text: queries::shield(game, card.id).to_string(),
                background_scale: 1.0,
            }),
            bottom_right_icon: definition
                .config
                .stats
                .base_attack
                .map(|_| CardIcon {
                    background: Some(assets::card_icon(CardIconType::Shield)),
                    text: queries::attack(game, card.id).to_string(),
                    background_scale: 1.0,
                })
                .or_else(|| {
                    definition.config.stats.health.map(|_| CardIcon {
                        background: Some(assets::card_icon(CardIconType::Health)),
                        text: queries::health(game, card.id).to_string(),
                        background_scale: 1.0,
                    })
                }),
            ..CardIcons::default()
        }
    } else {
        CardIcons {
            arena_icon: (card.data.card_level > 0).then(|| CardIcon {
                background: Some(assets::card_icon(CardIconType::LevelCounter)),
                text: card.data.card_level.to_string(),
                background_scale: 1.0,
            }),
            ..CardIcons::default()
        }
    }
}

/// Builds a [RevealedCardView], displaying a card for a user who can currently
/// see this card
fn revealed_card(
    game: &GameState,
    card: &CardState,
    definition: &CardDefinition,
    user_side: Side,
) -> RevealedCardView {
    RevealedCardView {
        card_frame: Some(assets::card_frame(definition.school)),
        title_background: Some(assets::title_background(definition.config.faction)),
        jewel: Some(assets::jewel(definition.rarity)),
        image: Some(sprite(&definition.image)),
        title: Some(CardTitle { text: definition.name.displayed_name() }),
        rules_text: Some(rules_text::build(game, card, definition)),
        revealed_in_arena: card.data.revealed,
        targeting: Some(card_targeting(definition)),
        on_release_position: Some(release_position(definition)),
        cost: Some(card_cost(game, user_side, card)),
        supplemental_info: None,
    }
}

/// Converts a card position into a rendered [ObjectPosition]. Returns None if
/// this [CardPosition] has no equivalent object position, e.g. if the card is
/// currently shuffled into the deck.
fn adapt_position(
    card: &CardState,
    position: CardPosition,
    user_side: Side,
) -> Option<ObjectPosition> {
    let result = match position {
        CardPosition::Room(room_id, location) => Some(Position::Room(ObjectPositionRoom {
            room_id: adapt_room_id(room_id).into(),
            room_location: match location {
                RoomLocation::Defender => ClientRoomLocation::Front,
                RoomLocation::InRoom => ClientRoomLocation::Back,
            }
            .into(),
        })),
        CardPosition::ArenaItem(location) => Some(Position::Item(ObjectPositionItem {
            item_location: match location {
                ItemLocation::Weapons => ClientItemLocation::Left,
                ItemLocation::Artifacts => ClientItemLocation::Right,
            }
            .into(),
        })),
        CardPosition::Hand(side) => Some(Position::Hand(ObjectPositionHand {
            owner: to_player_name(side, user_side).into(),
        })),
        CardPosition::DeckTop(side) => Some(Position::Deck(ObjectPositionDeck {
            owner: to_player_name(side, user_side).into(),
        })),
        CardPosition::DiscardPile(side) => Some(Position::DiscardPile(ObjectPositionDiscardPile {
            owner: to_player_name(side, user_side).into(),
        })),
        CardPosition::Scored(side) | CardPosition::Identity(side) => {
            Some(Position::Identity(ObjectPositionIdentity {
                owner: to_player_name(side, user_side).into(),
            }))
        }
        CardPosition::DeckUnknown(_side) => None,
    };

    result.map(|p| ObjectPosition { sorting_key: card.sorting_key, position: Some(p) })
}

/// Builds a description of the standard [CardTargeting] behavior of a card
fn card_targeting(definition: &CardDefinition) -> CardTargeting {
    CardTargeting {
        targeting: match definition.card_type {
            CardType::Spell | CardType::Weapon | CardType::Artifact | CardType::Identity => None,
            CardType::Minion | CardType::Project | CardType::Scheme | CardType::Upgrade => {
                Some(Targeting::PickRoom(PickRoom {}))
            }
        },
    }
}

/// Constructs the position to which a card should be moved once it is played
fn release_position(definition: &CardDefinition) -> ObjectPosition {
    ObjectPosition {
        sorting_key: u32::MAX,
        position: Some(match definition.card_type {
            CardType::Spell | CardType::Identity => Position::Staging(ObjectPositionStaging {}),
            CardType::Weapon => Position::Item(ObjectPositionItem {
                item_location: ClientItemLocation::Left.into(),
            }),
            CardType::Artifact => Position::Item(ObjectPositionItem {
                item_location: ClientItemLocation::Right.into(),
            }),
            CardType::Minion | CardType::Project | CardType::Scheme | CardType::Upgrade => {
                Position::Room(ObjectPositionRoom::default())
            }
        }),
    }
}

/// Constructs a delay command
fn delay(milliseconds: u32) -> Command {
    Command::Delay(DelayCommand { duration: Some(TimeValue { milliseconds }) })
}

/// Builds a structure describing a card's cost and whether it can currently be
/// played
fn card_cost(game: &GameState, user_side: Side, card: &CardState) -> CardViewCost {
    CardViewCost {
        mana_cost: queries::mana_cost(game, card.id).unwrap_or(0),
        action_cost: queries::action_cost(game, card.id),
        can_play: queries::can_play(game, user_side, card.id),
        can_play_algorithm: CanPlayAlgorithm::Optimistic.into(),
        spend_cost_algorithm: SpendCostAlgorithm::Optimistic.into(),
    }
}

/// Converts a [Side] into a [PlayerName] based on which viewer we are rendering
/// this update for.
fn to_player_name(side: Side, user_side: Side) -> PlayerName {
    if side == user_side {
        PlayerName::User
    } else {
        PlayerName::Opponent
    }
}

/// Converts a [CardId] into a client [GameObjectIdentifier]
fn adapt_game_object_id(id: CardId) -> GameObjectIdentifier {
    GameObjectIdentifier { id: Some(game_object_identifier::Id::CardId(adapt_card_id(id))) }
}

/// Turns a server [CardId] into its protobuf equivalent
pub fn adapt_card_id(card_id: CardId) -> CardIdentifier {
    CardIdentifier {
        side: match card_id.side {
            Side::Overlord => PlayerSide::Overlord,
            Side::Champion => PlayerSide::Champion,
        }
        .into(),
        index: card_id.index as u32,
    }
}

/// Turns a server [RoomId] into its protobuf equivalent
pub fn adapt_room_id(room_id: RoomId) -> RoomIdentifier {
    match room_id {
        RoomId::Vault => RoomIdentifier::Vault,
        RoomId::Sanctum => RoomIdentifier::Sanctum,
        RoomId::Crypts => RoomIdentifier::Crypts,
        RoomId::RoomA => RoomIdentifier::RoomA,
        RoomId::RoomB => RoomIdentifier::RoomB,
        RoomId::RoomC => RoomIdentifier::RoomC,
        RoomId::RoomD => RoomIdentifier::RoomD,
        RoomId::RoomE => RoomIdentifier::RoomE,
    }
}

/// Turns a [Sprite] into its protobuf equivalent
fn sprite(sprite: &Sprite) -> SpriteAddress {
    SpriteAddress { address: sprite.address.clone() }
}

/// Removes None values from a vector
fn filtered(vector: Vec<Option<Command>>) -> Vec<Command> {
    vector.into_iter().flatten().collect()
}
