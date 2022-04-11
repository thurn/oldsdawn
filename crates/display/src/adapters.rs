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

//! Helpers for converting between server & client representations

use anyhow::{anyhow, bail, Context, Result};
use data::primitives::{AbilityId, AbilityIndex, CardId, GameId, PlayerId, RoomId, Side};
use protos::spelldawn::game_object_identifier::Id;
use protos::spelldawn::{
    CardIdentifier, GameIdentifier, GameObjectIdentifier, PlayerIdentifier, PlayerName, PlayerSide,
    RoomIdentifier,
};

pub fn adapt_game_id(game_id: GameId) -> GameIdentifier {
    GameIdentifier { value: game_id.value }
}

/// Converts a [Side] into a [PlayerName] based on which viewer we are rendering
/// this update for.
pub fn to_player_name(side: Side, user_side: Side) -> PlayerName {
    if side == user_side {
        PlayerName::User
    } else {
        PlayerName::Opponent
    }
}

pub fn adapt_player_id(player_id: PlayerId) -> PlayerIdentifier {
    PlayerIdentifier { value: player_id.value }
}

pub fn adapt_side(side: Side) -> PlayerSide {
    match side {
        Side::Overlord => PlayerSide::Overlord,
        Side::Champion => PlayerSide::Champion,
    }
}

pub fn to_server_side(side: Option<PlayerSide>) -> Result<Side> {
    match side {
        Some(PlayerSide::Overlord) => Ok(Side::Overlord),
        Some(PlayerSide::Champion) => Ok(Side::Champion),
        _ => bail!("Invalid side"),
    }
}

/// Turns a server [CardId] into its protobuf equivalent
pub fn adapt_card_id(card_id: CardId) -> CardIdentifier {
    // TODO: Obfuscate this somehow, directly using the index leaks information
    CardIdentifier {
        side: match card_id.side {
            Side::Overlord => PlayerSide::Overlord,
            Side::Champion => PlayerSide::Champion,
        }
        .into(),
        index: card_id.index as u32,
        ability_id: None,
    }
}

/// Turns a server [CardId] and [AbilityIndex] into a protobuf equivalent
pub fn adapt_ability_card_id(card_id: CardId, ability_index: AbilityIndex) -> CardIdentifier {
    CardIdentifier { ability_id: Some(ability_index.value() as u32), ..adapt_card_id(card_id) }
}

/// Turns an [AbilityId] into a [CardIdentifier].
pub fn adapt_ability_id(ability_id: AbilityId) -> CardIdentifier {
    adapt_ability_card_id(ability_id.card_id, ability_id.index)
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

pub fn to_server_room_id(identifier: i32) -> Result<RoomId> {
    match RoomIdentifier::from_i32(identifier) {
        Some(RoomIdentifier::Vault) => Ok(RoomId::Vault),
        Some(RoomIdentifier::Sanctum) => Ok(RoomId::Sanctum),
        Some(RoomIdentifier::Crypts) => Ok(RoomId::Crypts),
        Some(RoomIdentifier::RoomA) => Ok(RoomId::RoomA),
        Some(RoomIdentifier::RoomB) => Ok(RoomId::RoomB),
        Some(RoomIdentifier::RoomC) => Ok(RoomId::RoomC),
        Some(RoomIdentifier::RoomD) => Ok(RoomId::RoomD),
        Some(RoomIdentifier::RoomE) => Ok(RoomId::RoomE),
        _ => bail!("Invalid RoomId: {:?}", identifier),
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum ServerCardId {
    CardId(CardId),
    AbilityId(AbilityId),
}

impl ServerCardId {
    // Helper for when it is an error for the value to not be a
    // `ServerCardId::CardId`.
    pub fn as_card_id(self) -> Result<CardId> {
        match self {
            ServerCardId::CardId(card_id) => Ok(card_id),
            ServerCardId::AbilityId(_) => bail!("Expected CardId"),
        }
    }
}

/// Converts a client [CardIdentifier] into a server [CardId] or [AbilityId].
pub fn to_server_card_id(card_id: Option<CardIdentifier>) -> Result<ServerCardId> {
    if let Some(id) = card_id {
        let result = CardId {
            side: match id.side() {
                PlayerSide::Overlord => Side::Overlord,
                PlayerSide::Champion => Side::Champion,
                _ => bail!("Invalid CardId {:?}", card_id),
            },
            index: id.index as usize,
        };

        id.ability_id.map_or(Ok(ServerCardId::CardId(result)), |index| {
            Ok(ServerCardId::AbilityId(AbilityId {
                card_id: result,
                index: AbilityIndex(index as usize),
            }))
        })
    } else {
        Err(anyhow!("Missing Required CardId"))
    }
}

/// Converts a [CardId] into a client [GameObjectIdentifier]
pub fn card_id_to_object_id(id: CardId) -> GameObjectIdentifier {
    GameObjectIdentifier { id: Some(Id::CardId(adapt_card_id(id))) }
}

/// Converts a client [PlayerIdentifier] into a server [PlayerId].
pub fn to_server_player_id(player_id: Option<PlayerIdentifier>) -> Result<PlayerId> {
    Ok(PlayerId::new(player_id.with_context(|| "PlayerId is required")?.value))
}
