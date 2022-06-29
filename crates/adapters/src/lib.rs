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

//! Helpers for converting between server & client data formats.

pub mod response_builder;

use anyhow::Result;
use data::fail;
use data::primitives::{
    AbilityId, AbilityIndex, CardId, GameId, GameObjectId, PlayerId, RoomId, Side, Sprite,
};
use protos::spelldawn::game_object_identifier::Id;
use protos::spelldawn::{
    CardIdentifier, GameIdentifier, GameObjectIdentifier, PlayerIdentifier, PlayerSide,
    RoomIdentifier, SpriteAddress, TimeValue,
};

use crate::response_builder::ResponseBuilder;

pub fn player_identifier(player_id: PlayerId) -> PlayerIdentifier {
    PlayerIdentifier { value: player_id.value }
}

pub fn player_id(player_id: PlayerIdentifier) -> PlayerId {
    PlayerId { value: player_id.value }
}

pub fn game_identifier(game_id: GameId) -> GameIdentifier {
    GameIdentifier { value: game_id.value }
}

pub fn game_id(game_id: GameIdentifier) -> GameId {
    GameId { value: game_id.value }
}

pub fn card_identifier(card_id: CardId) -> CardIdentifier {
    // Maybe need to obfuscate this somehow?
    CardIdentifier {
        side: player_side(card_id.side) as i32,
        index: card_id.index as u32,
        ability_id: None,
    }
}

pub fn game_object_identifier(
    builder: &ResponseBuilder,
    identifier: impl Into<GameObjectId>,
) -> GameObjectIdentifier {
    GameObjectIdentifier {
        id: Some(match identifier.into() {
            GameObjectId::CardId(card_id) => Id::CardId(card_identifier(card_id)),
            GameObjectId::AbilityId(ability_id) => Id::CardId(ability_card_identifier(ability_id)),
            GameObjectId::Deck(side) => Id::Deck(builder.to_player_name(side)),
            GameObjectId::DiscardPile(side) => Id::DiscardPile(builder.to_player_name(side)),
            GameObjectId::Identity(side) => Id::Identity(builder.to_player_name(side)),
        }),
    }
}

pub fn ability_card_identifier(ability_id: AbilityId) -> CardIdentifier {
    CardIdentifier {
        ability_id: Some(ability_id.index.value() as u32),
        ..card_identifier(ability_id.card_id)
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ServerCardId {
    CardId(CardId),
    AbilityId(AbilityId),
}

/// Converts a client [CardIdentifier] into a server [CardId] or [AbilityId].
pub fn server_card_id(card_id: CardIdentifier) -> Result<ServerCardId> {
    let result = CardId { side: side(card_id.side)?, index: card_id.index as usize };

    card_id.ability_id.map_or(Ok(ServerCardId::CardId(result)), |index| {
        Ok(ServerCardId::AbilityId(AbilityId {
            card_id: result,
            index: AbilityIndex(index as usize),
        }))
    })
}

pub fn player_side(side: Side) -> i32 {
    match side {
        Side::Overlord => PlayerSide::Overlord as i32,
        Side::Champion => PlayerSide::Champion as i32,
    }
}

pub fn side(side: i32) -> Result<Side> {
    match PlayerSide::from_i32(side) {
        Some(PlayerSide::Overlord) => Ok(Side::Overlord),
        Some(PlayerSide::Champion) => Ok(Side::Champion),
        _ => fail!("Invalid player side"),
    }
}

pub fn room_identifier(room_id: RoomId) -> i32 {
    (match room_id {
        RoomId::Vault => RoomIdentifier::Vault,
        RoomId::Sanctum => RoomIdentifier::Sanctum,
        RoomId::Crypts => RoomIdentifier::Crypts,
        RoomId::RoomA => RoomIdentifier::RoomA,
        RoomId::RoomB => RoomIdentifier::RoomB,
        RoomId::RoomC => RoomIdentifier::RoomC,
        RoomId::RoomD => RoomIdentifier::RoomD,
        RoomId::RoomE => RoomIdentifier::RoomE,
    }) as i32
}

pub fn room_id(identifier: i32) -> Result<RoomId> {
    match RoomIdentifier::from_i32(identifier) {
        Some(RoomIdentifier::Vault) => Ok(RoomId::Vault),
        Some(RoomIdentifier::Sanctum) => Ok(RoomId::Sanctum),
        Some(RoomIdentifier::Crypts) => Ok(RoomId::Crypts),
        Some(RoomIdentifier::RoomA) => Ok(RoomId::RoomA),
        Some(RoomIdentifier::RoomB) => Ok(RoomId::RoomB),
        Some(RoomIdentifier::RoomC) => Ok(RoomId::RoomC),
        Some(RoomIdentifier::RoomD) => Ok(RoomId::RoomD),
        Some(RoomIdentifier::RoomE) => Ok(RoomId::RoomE),
        _ => fail!("Invalid RoomId: {:?}", identifier),
    }
}

/// Turns a [Sprite] into its protobuf equivalent
pub fn sprite(sprite: &Sprite) -> SpriteAddress {
    SpriteAddress { address: sprite.address.clone() }
}

pub fn milliseconds(milliseconds: u32) -> TimeValue {
    TimeValue { milliseconds }
}
