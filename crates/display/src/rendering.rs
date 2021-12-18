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
use cards::queries;
use model::card_definition::CardDefinition;
use model::card_state::{CardPosition, CardState};
use model::game::GameState;
use model::primitives;
use model::primitives::{CardType, Side, Sprite};
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
    PickRoom, PlayerInfo, PlayerName, PlayerView, RevealedCardView, RoomId, RoomLocation,
    ScoreView, SpendCostAlgorithm, SpriteAddress, UpdateGameViewCommand,
};

/// Produces a [CommandList] representing required updates to the provided [GameState].
pub fn server_response(game: &GameState, user_side: Side) -> CommandList {
    let mut commands = vec![];

    if game.modified() {
        commands.push(command(Command::UpdateGameView(UpdateGameViewCommand {
            game: Some(game_view(game, user_side)),
        })));
    }

    for id in game.all_card_ids() {
        let card = game.card(id);

        if card.data_modified() {
            commands.push(command(Command::CreateOrUpdateCard(create_or_update_card(
                game, card, user_side,
            ))))
        }

        if card.position_modified() {
            commands.push(command(Command::MoveGameObjects(move_card(card, user_side, false))));
        }
    }

    CommandList { commands }
}

fn game_view(game: &GameState, user_side: Side) -> GameView {
    GameView {
        game_id: Some(adapt_game_id(game.id())),
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
    }
}

fn move_card(card: &CardState, user_side: Side, disable_animation: bool) -> MoveGameObjectsCommand {
    MoveGameObjectsCommand {
        ids: vec![adapt_game_object_id(card.id())],
        position: Some(adapt_position(card.position(), user_side)),
        index: None,
        disable_animation,
    }
}

fn create_or_update_card(
    game: &GameState,
    card: &CardState,
    user_side: Side,
) -> CreateOrUpdateCardCommand {
    let definition = cards::get(card.name());
    let revealed = definition.side == user_side || card.data().revealed;
    CreateOrUpdateCardCommand {
        card: Some(CardView {
            card_id: Some(adapt_card_id(card.id())),
            card_icons: Some(card_icons(game, card, definition, revealed)),
            arena_frame: Some(assets::arena_frame(
                definition.side,
                definition.card_type,
                definition.config.faction,
            )),
            owning_player: to_player_name(definition.side, user_side).into(),
            revealed_card: if revealed {
                Some(revealed_card(game, card, definition, user_side))
            } else {
                None
            },
        }),
        create_position: Some(adapt_position(card.position(), user_side)),
        create_animation: CardCreationAnimation::Unspecified.into(),
        disable_flip_animation: false,
    }
}

fn player_view(game: &GameState, side: Side) -> PlayerView {
    let identity = game.identity(side);
    let data = game.player(side);
    PlayerView {
        player_info: Some(PlayerInfo {
            name: identity.name().displayed_name(),
            portrait: Some(sprite(&cards::get(identity.name()).image)),
            portrait_frame: Some(assets::identity_card_frame(side)),
            card_back: Some(assets::card_back(cards::get(identity.name()).school)),
        }),
        score: Some(ScoreView { score: data.score }),
        mana: Some(ManaView { amount: data.mana }),
        action_tracker: Some(ActionTrackerView { available_action_count: data.actions }),
    }
}

fn current_priority(game: &GameState, user_side: Side) -> PlayerName {
    to_player_name(
        match game.data().raid {
            Some(raid) => raid.priority,
            None => game.data().turn,
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
            top_left_icon: queries::mana_cost(game, card.id()).map(|mana| CardIcon {
                background: Some(assets::card_icon(CardIconType::Mana)),
                text: mana.to_string(),
                background_scale: 1.0,
            }),
            bottom_left_icon: definition.config.stats.shield.map(|_| CardIcon {
                background: Some(assets::card_icon(CardIconType::Shield)),
                text: queries::shield(game, card.id()).to_string(),
                background_scale: 1.0,
            }),
            bottom_right_icon: definition
                .config
                .stats
                .base_attack
                .map(|_| CardIcon {
                    background: Some(assets::card_icon(CardIconType::Shield)),
                    text: queries::attack(game, card.id()).to_string(),
                    background_scale: 1.0,
                })
                .or_else(|| {
                    definition.config.stats.health.map(|_| CardIcon {
                        background: Some(assets::card_icon(CardIconType::Health)),
                        text: queries::health(game, card.id()).to_string(),
                        background_scale: 1.0,
                    })
                }),
            ..CardIcons::default()
        }
    } else {
        CardIcons {
            arena_icon: if card.data().card_level > 0 {
                Some(CardIcon {
                    background: Some(assets::card_icon(CardIconType::LevelCounter)),
                    text: card.data().card_level.to_string(),
                    background_scale: 1.0,
                })
            } else {
                None
            },
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
        revealed_in_arena: card.data().revealed,
        targeting: Some(card_targeting(definition)),
        on_release_position: Some(release_position(definition)),
        cost: Some(card_cost(game, card)),
        supplemental_info: None,
    }
}

fn adapt_position(position: CardPosition, user_side: Side) -> ObjectPosition {
    ObjectPosition {
        position: Some(match position {
            CardPosition::Room(room_id, location) => Position::Room(ObjectPositionRoom {
                room_id: adapt_room_id(room_id).into(),
                room_location: match location {
                    primitives::RoomLocation::Defender(_) => RoomLocation::Front,
                    primitives::RoomLocation::InRoom => RoomLocation::Back,
                }
                .into(),
                index: match location {
                    primitives::RoomLocation::Defender(position) => Some(position),
                    primitives::RoomLocation::InRoom => None,
                },
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
            CardPosition::Deck(side) => {
                Position::Deck(ObjectPositionDeck { owner: to_player_name(side, user_side).into() })
            }
            CardPosition::DiscardPile(side) => Position::DiscardPile(ObjectPositionDiscardPile {
                owner: to_player_name(side, user_side).into(),
            }),
            CardPosition::Scored(side) => Position::Identity(ObjectPositionIdentity {
                owner: to_player_name(side, user_side).into(),
            }),
            CardPosition::Identity(side) => {
                Position::IdentityContainer(ObjectPositionIdentityContainer {
                    owner: to_player_name(side, user_side).into(),
                })
            }
        }),
    }
}

fn card_targeting(definition: &CardDefinition) -> CardTargeting {
    CardTargeting {
        targeting: match definition.card_type {
            CardType::Spell => None,
            CardType::Weapon => None,
            CardType::Artifact => None,
            CardType::Minion => Some(Targeting::PickRoom(PickRoom {})),
            CardType::Project => Some(Targeting::PickRoom(PickRoom {})),
            CardType::Scheme => Some(Targeting::PickRoom(PickRoom {})),
            CardType::Upgrade => Some(Targeting::PickRoom(PickRoom {})),
            CardType::Identity => None,
            CardType::Token => None,
        },
    }
}

fn release_position(definition: &CardDefinition) -> ObjectPosition {
    ObjectPosition {
        position: Some(match definition.card_type {
            CardType::Spell => Position::Staging(ObjectPositionStaging {}),
            CardType::Weapon => {
                Position::Item(ObjectPositionItem { item_location: ItemLocation::Left.into() })
            }
            CardType::Artifact => {
                Position::Item(ObjectPositionItem { item_location: ItemLocation::Right.into() })
            }
            CardType::Minion => Position::Room(ObjectPositionRoom::default()),
            CardType::Project => Position::Room(ObjectPositionRoom::default()),
            CardType::Scheme => Position::Room(ObjectPositionRoom::default()),
            CardType::Upgrade => Position::Room(ObjectPositionRoom::default()),
            CardType::Identity => Position::Staging(ObjectPositionStaging {}),
            CardType::Token => Position::Staging(ObjectPositionStaging {}),
        }),
    }
}

fn card_cost(game: &GameState, card: &CardState) -> CardCost {
    CardCost {
        mana_cost: queries::mana_cost(game, card.id()).unwrap_or(0),
        action_cost: queries::action_cost(game, card.id()),
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

fn adapt_game_id(string: &str) -> GameId {
    GameId { value: string.to_string() }
}

fn adapt_game_object_id(id: primitives::CardId) -> GameObjectId {
    GameObjectId { id: Some(game_object_id::Id::CardId(adapt_card_id(id))) }
}

fn adapt_card_id(card_id: primitives::CardId) -> CardId {
    CardId { side: card_id.side as u32, index: card_id.index as u32 }
}

fn adapt_room_id(room_id: primitives::RoomId) -> RoomId {
    match room_id {
        primitives::RoomId::Treasury => RoomId::Treasury,
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
