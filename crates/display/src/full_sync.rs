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

use std::collections::HashMap;

use data::card_definition::CardDefinition;
use data::card_state::{CardPosition, CardPositionKind, CardState};
use data::game::GameState;
use data::primitives::{CardId, CardType, ItemLocation, RoomId, RoomLocation, Side, Sprite};
use protos::spelldawn::card_targeting::Targeting;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{
    identity_action, ActionTrackerView, ArenaView, CardCreationAnimation, CardIcon, CardIcons,
    CardIdentifier, CardTargeting, CardTitle, CardView, ClientItemLocation, ClientRoomLocation,
    CreateOrUpdateCardCommand, GameIdentifier, GameView, IdentityAction, ManaView, ObjectPosition,
    ObjectPositionDeck, ObjectPositionDiscardPile, ObjectPositionHand, ObjectPositionIdentity,
    ObjectPositionItem, ObjectPositionRoom, ObjectPositionStaging, PickRoom, PlayerInfo,
    PlayerName, PlayerSide, PlayerView, RevealedCardView, RoomIdentifier, ScoreView, SpriteAddress,
    UpdateGameViewCommand,
};
use rules::{flags, queries};

use crate::assets::CardIconType;
use crate::{assets, rules_text};

pub struct FullSync {
    pub game: UpdateGameViewCommand,
    pub cards: HashMap<CardId, CreateOrUpdateCardCommand>,
}

pub fn run(game: &GameState, user_side: Side) -> FullSync {
    FullSync {
        game: update_game_view(game, user_side),
        cards: game
            .all_cards()
            .filter(|c| c.position.kind() != CardPositionKind::DeckUnknown)
            .map(|c| {
                (
                    c.id,
                    create_or_update_card(
                        game,
                        c,
                        user_side,
                        CardCreationStrategy::SnapToCurrentPosition,
                    ),
                )
            })
            .collect(),
    }
}

/// Builds a command to update the client's current [GameView]
fn update_game_view(game: &GameState, user_side: Side) -> UpdateGameViewCommand {
    UpdateGameViewCommand {
        game: Some(GameView {
            game_id: Some(GameIdentifier { value: game.id.value }),
            user: Some(player_view(game, user_side)),
            opponent: Some(player_view(game, user_side.opponent())),
            arena: Some(ArenaView {
                rooms_at_bottom: Some(user_side == Side::Overlord),
                identity_action: Some(match user_side {
                    Side::Overlord => IdentityAction {
                        identity_action: Some(identity_action::IdentityAction::LevelUpRoom(())),
                    },
                    Side::Champion => IdentityAction {
                        identity_action: Some(identity_action::IdentityAction::InitiateRaid(())),
                    },
                }),
            }),
            current_priority: current_priority(game, user_side).into(),
        }),
    }
}

/// Builds a [PlayerView] for a given player
fn player_view(game: &GameState, side: Side) -> PlayerView {
    let identity = game.identity(side);
    let data = game.player(side);
    PlayerView {
        player_info: Some(PlayerInfo {
            name: Some(identity.name.displayed_name()),
            portrait: Some(sprite(&rules::get(identity.name).image)),
            portrait_frame: Some(assets::identity_card_frame(side)),
            card_back: Some(assets::card_back(rules::get(identity.name).school)),
        }),
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

/// Possible behavior when creating a card
#[derive(Debug, PartialEq, Clone)]
pub enum CardCreationStrategy {
    /// Animate the card moving from the user's deck to the staging area.
    DrawUserCard,
    /// Jump the newly-created card to its current game position. If the current
    /// position is invalid (e.g. in the user's deck), no initial position
    /// will be specified for the card.
    SnapToCurrentPosition,
    /// Create the card at a specific game object position.
    CreateAtPosition(ObjectPosition),
}

/// Creates a command to create or update a card.
pub fn create_or_update_card(
    game: &GameState,
    card: &CardState,
    user_side: Side,
    creation_strategy: CardCreationStrategy,
) -> CreateOrUpdateCardCommand {
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
            adapt_position(card, card.position, user_side)
        }
        CardCreationStrategy::CreateAtPosition(p) => Some(p),
    };

    CreateOrUpdateCardCommand {
        card: Some(CardView {
            card_id: Some(adapt_card_id(card.id)),
            card_icons: Some(card_icons(game, card, definition, revealed)),
            arena_frame: Some(assets::arena_frame(
                definition.side,
                definition.card_type,
                definition.config.faction,
            )),
            owning_player: to_player_name(definition.side, user_side).into(),
            revealed_card: revealed.then(|| revealed_card_view(game, card, definition, user_side)),
        }),
        create_position: position,
        create_animation,
        disable_flip_animation: false,
    }
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
                text: Some(mana.to_string()),
                background_scale: 1.0,
            }),
            bottom_left_icon: definition.config.stats.shield.map(|_| CardIcon {
                background: Some(assets::card_icon(CardIconType::Shield)),
                text: Some(queries::shield(game, card.id).to_string()),
                background_scale: 1.0,
            }),
            bottom_right_icon: definition
                .config
                .stats
                .base_attack
                .map(|_| CardIcon {
                    background: Some(assets::card_icon(CardIconType::Shield)),
                    text: Some(queries::attack(game, card.id).to_string()),
                    background_scale: 1.0,
                })
                .or_else(|| {
                    definition.config.stats.health.map(|_| CardIcon {
                        background: Some(assets::card_icon(CardIconType::Health)),
                        text: Some(queries::health(game, card.id).to_string()),
                        background_scale: 1.0,
                    })
                }),
            ..CardIcons::default()
        }
    } else {
        CardIcons {
            arena_icon: (card.data.card_level > 0).then(|| CardIcon {
                background: Some(assets::card_icon(CardIconType::LevelCounter)),
                text: Some(card.data.card_level.to_string()),
                background_scale: 1.0,
            }),
            ..CardIcons::default()
        }
    }
}

/// Builds a [RevealedCardView], displaying a card for a user who can currently
/// see this card
fn revealed_card_view(
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
        can_play: flags::can_take_play_card_action(game, user_side, card.id),
        supplemental_info: None,
    }
}

/// Converts a card position into a rendered [ObjectPosition]. Returns None if
/// this [CardPosition] has no equivalent object position, e.g. if the card is
/// currently shuffled into the deck.
pub fn adapt_position(
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
            CardType::Minion => Position::Room(ObjectPositionRoom {
                room_id: RoomIdentifier::Unspecified.into(),
                room_location: ClientRoomLocation::Front.into(),
            }),
            CardType::Project | CardType::Scheme | CardType::Upgrade => {
                Position::Room(ObjectPositionRoom {
                    room_id: RoomIdentifier::Unspecified.into(),
                    room_location: ClientRoomLocation::Back.into(),
                })
            }
        }),
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

/// Turns a [Sprite] into its protobuf equivalent
fn sprite(sprite: &Sprite) -> SpriteAddress {
    SpriteAddress { address: sprite.address.clone() }
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
