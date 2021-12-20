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

use crate::assets::{jewel, CardIconType};
use crate::{assets, rules_text};
use data::card_definition::CardDefinition;
use data::card_state::{CardPosition, CardPositionKind, CardState};
use data::game::GameState;
use data::primitives;
use data::primitives::{CardType, Side, Sprite};
use protos::spelldawn::card_targeting::Targeting;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{
    game_object_id, ActionTrackerView, ArenaView, CanPlayAlgorithm, CardCost,
    CardCreationAnimation, CardIcon, CardIcons, CardId, CardTargeting, CardTitle, CardView,
    CommandList, CreateOrUpdateCardCommand, GameCommand, GameId, GameObjectId, GameView,
    IdentityAction, ItemLocation, ManaView, MoveGameObjectsCommand, ObjectPosition,
    ObjectPositionDeck, ObjectPositionDiscardPile, ObjectPositionHand, ObjectPositionIdentity,
    ObjectPositionIdentityContainer, ObjectPositionItem, ObjectPositionRoom, ObjectPositionStaging,
    PickRoom, PlayerInfo, PlayerName, PlayerSide, PlayerView, RevealedCardView, RoomId,
    RoomLocation, ScoreView, SpendCostAlgorithm, SpriteAddress, UpdateGameViewCommand,
};
use rules::queries;

/// Builds a series of [GameCommand]s to fully represent the current state of this game in the
/// client, for use e.g. in response to a reconnect request.
pub fn full_sync(game: &GameState, user_side: Side) -> Vec<GameCommand> {
    let mut result = vec![game_view(game, user_side)];
    result.extend(
        game.all_cards()
            .filter(|c| c.position.kind() != CardPositionKind::DeckUnknown)
            .map(|c| create_or_update_card(game, c, user_side)),
    );
    result
}

fn game_view(game: &GameState, user_side: Side) -> GameCommand {
    GameCommand {
        command: Some(Command::UpdateGameView(UpdateGameViewCommand {
            game: Some(GameView {
                game_id: Some(GameId { value: game.id.value }),
                user: Some(player_view(game, user_side)),
                opponent: Some(player_view(game, user_side.opponent())),
                arena: Some(ArenaView {
                    rooms_at_bottom: Some(user_side == Side::Overlord),
                    identity_action: match user_side {
                        Side::Overlord => IdentityAction::LevelUpRoom.into(),
                        Side::Champion => IdentityAction::InitiateRaid.into(),
                    },
                }),
                current_priority: current_priority(game, user_side).into(),
            }),
        })),
    }
}

/// Creates a move card command to move a card to its current location. Panics if this card isn't
/// in a valid game position, e.g. if it is in [CardPosition::DeckUnknown].
fn move_card(
    game: &GameState,
    card: &CardState,
    user_side: Side,
    disable_animation: bool,
) -> MoveGameObjectsCommand {
    MoveGameObjectsCommand {
        ids: vec![adapt_game_object_id(card.id)],
        position: Some(adapt_position(card, game.card(card.id).position, user_side)),
        disable_animation,
    }
}

/// Creates a create/update card command. Panics if this card isn't in a valid game position, e.g
/// if it is in [CardPosition::DeckUnknown].
fn create_or_update_card(game: &GameState, card: &CardState, user_side: Side) -> GameCommand {
    let definition = rules::get(card.name);
    let revealed = definition.side == user_side || card.data.revealed;

    GameCommand {
        command: Some(Command::CreateOrUpdateCard(CreateOrUpdateCardCommand {
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
            create_position: Some(adapt_position(card, game.card(card.id).position, user_side)),
            create_animation: CardCreationAnimation::Unspecified.into(),
            disable_flip_animation: false,
        })),
    }
}

fn player_view(game: &GameState, side: Side) -> PlayerView {
    let identity = game.identity(side);
    let data = game.player(side);
    PlayerView {
        player_info: Some(PlayerInfo {
            name: identity.name.displayed_name(),
            portrait: Some(sprite(&rules::get(identity.name).image)),
            portrait_frame: Some(assets::identity_card_frame(side)),
            card_back: Some(assets::card_back(rules::get(identity.name).school)),
        }),
        score: Some(ScoreView { score: data.score }),
        mana: Some(ManaView { amount: data.mana }),
        action_tracker: Some(ActionTrackerView { available_action_count: data.actions }),
    }
}

fn current_priority(game: &GameState, user_side: Side) -> PlayerName {
    to_player_name(
        match game.data.raid {
            Some(raid) => raid.priority,
            None => game.data.turn,
        },
        user_side,
    )
}

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

fn revealed_card(
    game: &GameState,
    card: &CardState,
    definition: &CardDefinition,
    user_side: Side,
) -> RevealedCardView {
    RevealedCardView {
        card_frame: Some(assets::card_frame(definition.school)),
        title_background: Some(assets::title_background(definition.config.faction)),
        jewel: Some(jewel(definition.rarity)),
        image: Some(sprite(&definition.image)),
        title: Some(CardTitle { text: definition.name.displayed_name() }),
        rules_text: Some(rules_text::build(game, card, definition, user_side)),
        revealed_in_arena: card.data.revealed,
        targeting: Some(card_targeting(definition)),
        on_release_position: Some(release_position(definition)),
        cost: Some(card_cost(game, card)),
        supplemental_info: None,
    }
}

/// Converts a card position into a rendered [ObjectPosition]. Panics if this [CardPosition] has no
/// equivalent object position, e.g. if the card is currently shuffled into the deck.
fn adapt_position(card: &CardState, position: CardPosition, user_side: Side) -> ObjectPosition {
    ObjectPosition {
        sorting_key: Some(card.sorting_key),
        position: Some(match position {
            CardPosition::Room(room_id, location) => Position::Room(ObjectPositionRoom {
                room_id: adapt_room_id(room_id).into(),
                room_location: match location {
                    primitives::RoomLocation::Defender => RoomLocation::Front,
                    primitives::RoomLocation::InRoom => RoomLocation::Back,
                }
                .into(),
            }),
            CardPosition::ArenaItem(location) => Position::Item(ObjectPositionItem {
                item_location: match location {
                    primitives::ItemLocation::Weapons => ItemLocation::Left,
                    primitives::ItemLocation::Artifacts => ItemLocation::Right,
                }
                .into(),
            }),
            CardPosition::Hand(side) => {
                Position::Hand(ObjectPositionHand { owner: to_player_name(side, user_side).into() })
            }
            CardPosition::DeckTop(side) => {
                Position::Deck(ObjectPositionDeck { owner: to_player_name(side, user_side).into() })
            }
            CardPosition::DiscardPile(side) => Position::DiscardPile(ObjectPositionDiscardPile {
                owner: to_player_name(side, user_side).into(),
            }),
            CardPosition::Scored(side) | CardPosition::Identity(side) => {
                Position::Identity(ObjectPositionIdentity {
                    owner: to_player_name(side, user_side).into(),
                })
            }
            CardPosition::DeckUnknown(side) => panic!("Card has no game position."),
        }),
    }
}

fn card_targeting(definition: &CardDefinition) -> CardTargeting {
    CardTargeting {
        targeting: match definition.card_type {
            CardType::Spell
            | CardType::Weapon
            | CardType::Artifact
            | CardType::Identity
            | CardType::Token => None,
            CardType::Minion | CardType::Project | CardType::Scheme | CardType::Upgrade => {
                Some(Targeting::PickRoom(PickRoom {}))
            }
        },
    }
}

fn release_position(definition: &CardDefinition) -> ObjectPosition {
    ObjectPosition {
        sorting_key: None,
        position: Some(match definition.card_type {
            CardType::Spell | CardType::Identity | CardType::Token => {
                Position::Staging(ObjectPositionStaging {})
            }
            CardType::Weapon => {
                Position::Item(ObjectPositionItem { item_location: ItemLocation::Left.into() })
            }
            CardType::Artifact => {
                Position::Item(ObjectPositionItem { item_location: ItemLocation::Right.into() })
            }
            CardType::Minion | CardType::Project | CardType::Scheme | CardType::Upgrade => {
                Position::Room(ObjectPositionRoom::default())
            }
        }),
    }
}

fn card_cost(game: &GameState, card: &CardState) -> CardCost {
    CardCost {
        mana_cost: queries::mana_cost(game, card.id).unwrap_or(0),
        action_cost: queries::action_cost(game, card.id),
        can_play: false,
        can_play_algorithm: CanPlayAlgorithm::Optimistic.into(),
        spend_cost_algorithm: SpendCostAlgorithm::Optimistic.into(),
    }
}

fn to_player_name(side: Side, user_side: Side) -> PlayerName {
    if side == user_side {
        PlayerName::User
    } else {
        PlayerName::Opponent
    }
}

fn command(command: Command) -> GameCommand {
    GameCommand { command: Some(command) }
}

fn adapt_game_object_id(id: primitives::CardId) -> GameObjectId {
    GameObjectId { id: Some(game_object_id::Id::CardId(adapt_card_id(id))) }
}

fn adapt_card_id(card_id: primitives::CardId) -> CardId {
    CardId {
        side: match card_id.side {
            Side::Overlord => PlayerSide::Overlord,
            Side::Champion => PlayerSide::Champion,
        }
        .into(),
        index: card_id.index as u32,
    }
}

fn adapt_room_id(room_id: primitives::RoomId) -> RoomId {
    match room_id {
        primitives::RoomId::Vault => RoomId::Vault,
        primitives::RoomId::Sanctum => RoomId::Sanctum,
        primitives::RoomId::Crypts => RoomId::Crypts,
        primitives::RoomId::RoomA => RoomId::RoomA,
        primitives::RoomId::RoomB => RoomId::RoomB,
        primitives::RoomId::RoomC => RoomId::RoomC,
        primitives::RoomId::RoomD => RoomId::RoomD,
        primitives::RoomId::RoomE => RoomId::RoomE,
    }
}

fn sprite(sprite: &Sprite) -> SpriteAddress {
    SpriteAddress { address: sprite.address.clone() }
}
