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

use anyhow::Result;
use data::card_state::{CardPosition, CardState};
use data::fail;
use data::game::{GamePhase, GameState, MulliganData, RaidData, RaidPhase};
use data::primitives::{AbilityId, CardId, ItemLocation, RoomId, RoomLocation, Side};
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{ClientItemLocation, ClientRoomLocation, ObjectPosition, ObjectPositionBrowser, ObjectPositionDeck, ObjectPositionDiscardPile, ObjectPositionHand, ObjectPositionIdentity, ObjectPositionIntoCard, ObjectPositionItem, ObjectPositionRaid, ObjectPositionRevealedCards, ObjectPositionRoom, ObjectPositionStaging};

use crate::adapters;
use crate::response_builder::ResponseBuilder;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum GameObjectId {
    CardId(CardId),
    AbilityId(AbilityId),
    Deck(Side),
    DiscardPile(Side),
    Identity(Side),
}

impl From<CardId> for GameObjectId {
    fn from(card_id: CardId) -> Self {
        GameObjectId::CardId(card_id)
    }
}

impl From<AbilityId> for GameObjectId {
    fn from(ability_id: AbilityId) -> Self {
        GameObjectId::AbilityId(ability_id)
    }
}

pub fn for_card(card: &CardState, position: Position) -> ObjectPosition {
    ObjectPosition { position: Some(position), sorting_key: card.sorting_key, sorting_subkey: 0 }
}

pub fn for_ability(game: &GameState, ability_id: AbilityId, position: Position) -> ObjectPosition {
    ObjectPosition {
        position: Some(position),
        sorting_key: game.card(ability_id.card_id).sorting_key,
        sorting_subkey: 1 + (ability_id.index.value() as u32),
    }
}

pub fn for_sorting_key(
    sorting_key: u32,
    position: Position,
) -> ObjectPosition {
    ObjectPosition { sorting_key, sorting_subkey: 0, position: Some(position) }
}

pub fn room(room_id: RoomId, location: RoomLocation) -> Position {
    Position::Room(ObjectPositionRoom {
        room_id: adapters::room_identifier(room_id),
        room_location: match location {
            RoomLocation::Defender => ClientRoomLocation::Front,
            RoomLocation::Occupant => ClientRoomLocation::Back,
        }
        .into(),
    })
}

pub fn item(location: ItemLocation) -> Position {
    Position::Item(ObjectPositionItem {
        item_location: match location {
            ItemLocation::Weapons => ClientItemLocation::Left,
            ItemLocation::Artifacts => ClientItemLocation::Right,
        }
        .into(),
    })
}

pub fn hand(builder: &ResponseBuilder, side: Side) -> Position {
    Position::Hand(ObjectPositionHand { owner: builder.to_player_name(side) })
}

pub fn deck_top(builder: &ResponseBuilder, side: Side) -> Position {
    Position::Deck(ObjectPositionDeck { owner: builder.to_player_name(side) })
}

pub fn discard(builder: &ResponseBuilder, side: Side) -> Position {
    Position::DiscardPile(ObjectPositionDiscardPile { owner: builder.to_player_name(side) })
}

pub fn scored(builder: &ResponseBuilder, side: Side) -> Position {
    Position::Identity(ObjectPositionIdentity { owner: builder.to_player_name(side) })
}

pub fn staging() -> Position {
    Position::Staging(ObjectPositionStaging {})
}

pub fn browser() -> Position {
    Position::Browser(ObjectPositionBrowser {})
}

pub fn revealed_cards() -> Position {
    Position::Revealed(ObjectPositionRevealedCards {})
}

pub fn raid() -> Position {
    Position::Raid(ObjectPositionRaid {})
}

pub fn parent_card(ability_id: AbilityId) -> Position {
    Position::IntoCard(ObjectPositionIntoCard {
        card_id: Some(adapters::card_identifier(ability_id.card_id)),
    })
}

pub fn convert(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
) -> Result<ObjectPosition> {
    Ok(if let Some(position_override) = position_override(builder, game, card)? {
        position_override
    } else {
        ObjectPosition {
            sorting_key: card.sorting_key,
            position: Some(match card.position() {
                CardPosition::Room(room_id, location) => room(room_id, location),
                CardPosition::ArenaItem(location) => item(location),
                CardPosition::Hand(side) => hand(builder, side),
                CardPosition::DeckTop(side) => deck_top(builder, side),
                CardPosition::DiscardPile(side) => discard(builder, side),
                CardPosition::Scored(side) | CardPosition::Identity(side) => scored(builder, side),
                CardPosition::DeckUnknown(_) => fail!("Invalid card position"),
            }),
            ..ObjectPosition::default()
        }
    })
}

fn position_override(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
) -> Result<Option<ObjectPosition>> {
    match &game.data.phase {
        GamePhase::ResolveMulligans(mulligans) => {
            Ok(opening_hand_position_override(builder, game, card, mulligans))
        }
        GamePhase::Play => Ok(if let Some(raid) = &game.data.raid {
            if raid.phase == RaidPhase::Access {
                browser_position(card, browser(), raid_access_browser(game, raid))
            } else {
                browser_position(card, browser(), raid_browser(game, raid)?)
            }
        } else {
            None
        }),
        _ => Ok(None),
    }
}

fn opening_hand_position_override(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
    data: &MulliganData,
) -> Option<ObjectPosition> {
    if data.decision(builder.user_side).is_none()
        && game.hand(builder.user_side).any(|c| c.id == card.id)
    {
        Some(for_card(card, browser()))
    } else {
        None
    }
}

fn browser_position(
    card: &CardState,
    position: Position,
    browser: Vec<GameObjectId>,
) -> Option<ObjectPosition> {
    browser
        .iter()
        .position(|gid| matches!(gid, GameObjectId::CardId(card_id) if *card_id == card.id))
        .map(|index| ObjectPosition {
            sorting_key: index as u32,
            sorting_subkey: 0,
            position: Some(position),
        })
}

fn raid_browser(game: &GameState, raid: &RaidData) -> Result<Vec<GameObjectId>> {
    let mut result = Vec::new();

    match raid.target {
        RoomId::Vault => {
            result.push(GameObjectId::Deck(Side::Overlord));
        }
        RoomId::Sanctum => {
            result.push(GameObjectId::Identity(Side::Overlord));
        }
        RoomId::Crypts => {
            result.push(GameObjectId::DiscardPile(Side::Overlord));
        }
        _ => {}
    }

    result.extend(game.occupants(raid.target).map(|card| GameObjectId::CardId(card.id)));

    let defenders = game.defender_list(raid.target);
    let included = match raid.phase {
        RaidPhase::Activation => &defenders,
        RaidPhase::Encounter(i) => &defenders[..=i],
        RaidPhase::Continue(i) => &defenders[..=i],
        _ => fail!("Expected raid to be in-flight"),
    };
    result.extend(included.iter().map(|card_id| GameObjectId::CardId(*card_id)));
    result.push(GameObjectId::Identity(Side::Champion));

    Ok(result)
}

fn raid_access_browser(game: &GameState, raid: &RaidData) -> Vec<GameObjectId> {
    match raid.target {
        RoomId::Sanctum => {
            game.hand(Side::Overlord).map(|card| GameObjectId::CardId(card.id)).collect()
        }
        RoomId::Crypts => {
            game.discard_pile(Side::Overlord).map(|card| GameObjectId::CardId(card.id)).collect()
        }
        _ => raid.accessed.iter().map(|card_id| GameObjectId::CardId(*card_id)).collect(),
    }
}
