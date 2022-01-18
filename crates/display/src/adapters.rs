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

use data::primitives::{CardId, RoomId, Side};
use protos::spelldawn::game_object_identifier::Id;
use protos::spelldawn::{
    CardIdentifier, GameObjectIdentifier, PlayerName, PlayerSide, RoomIdentifier,
};

use crate::full_sync::GameObjectId;

/// Converts a [Side] into a [PlayerName] based on which viewer we are rendering
/// this update for.
pub fn to_player_name(side: Side, user_side: Side) -> PlayerName {
    if side == user_side {
        PlayerName::User
    } else {
        PlayerName::Opponent
    }
}

pub fn adapt_side(side: Side) -> PlayerSide {
    match side {
        Side::Overlord => PlayerSide::Overlord,
        Side::Champion => PlayerSide::Champion,
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
    }
}

/// Turns a server [GameObjectId] into its protobuf equivalent
pub fn adapt_game_object_id(game_object_id: GameObjectId) -> GameObjectIdentifier {
    GameObjectIdentifier {
        id: Some(match game_object_id {
            GameObjectId::CardId(card_id) => Id::CardId(adapt_card_id(card_id)),
            GameObjectId::Identity(name) => Id::Identity(name.into()),
            GameObjectId::Deck(name) => Id::Deck(name.into()),
            GameObjectId::DiscardPile(name) => Id::DiscardPile(name.into()),
        }),
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
