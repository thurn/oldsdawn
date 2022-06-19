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

//! Defines the state of cards during an ongoing game.

#![allow(clippy::use_self)] // Required to use EnumKind

use std::cmp::Ordering;
use std::collections::HashMap;

use enum_kinds::EnumKind;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::card_name::CardName;
use crate::game::TurnData;
use crate::primitives::{
    BoostCount, CardId, ItemLocation, LevelValue, ManaValue, RaidId, RoomId, RoomLocation, Side,
};

/// State for an ability within a game
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde_as]
pub struct AbilityState {
    /// True if this ability is currently being resolved
    pub on_stack: bool,
    pub raid_id: Option<RaidId>,
    pub turn: Option<TurnData>,
    #[serde_as(as = "Vec<(_, _)>")]
    pub room_turns: HashMap<RoomId, TurnData>,
}

/// Identifies the location of a card during an active game
#[derive(
    PartialEq, Eq, Hash, Debug, Copy, Clone, EnumKind, Serialize, Deserialize, Ord, PartialOrd,
)]
#[enum_kind(CardPositionKind)]
pub enum CardPosition {
    /// An unspecified random position within a user's deck. The default
    /// position of all cards when a new game is started.
    DeckUnknown(Side),
    /// A card which is known to at least one player to be on the top of a deck
    DeckTop(Side),
    Hand(Side),
    Room(RoomId, RoomLocation),
    ArenaItem(ItemLocation),
    DiscardPile(Side),
    Scored(Side),
    /// A card is in the process of resolving
    Stack,
    /// Marks the identity card for a side. The first identity (by sorting key)
    /// is the primary identity for a player.
    Identity(Side),
}

impl CardPosition {
    /// Returns the [CardPositionKind] for this card
    pub fn kind(&self) -> CardPositionKind {
        self.into()
    }

    /// Returns true if this card is in a room or has been played as an item
    pub fn in_play(&self) -> bool {
        matches!(self.kind(), CardPositionKind::Room | CardPositionKind::ArenaItem)
    }

    /// Returns true if this card is in a user's hand
    pub fn in_hand(&self) -> bool {
        self.kind() == CardPositionKind::Hand
    }

    // True if a card is currently shuffled into a deck
    pub fn shuffled_into_deck(&self) -> bool {
        self.kind() == CardPositionKind::DeckUnknown
    }

    /// Returns true if this card is in a known or unknown deck position
    pub fn in_deck(&self) -> bool {
        matches!(self.kind(), CardPositionKind::DeckUnknown | CardPositionKind::DeckTop)
    }

    /// Returns true if this card is in a user's discard pile
    pub fn in_discard_pile(&self) -> bool {
        self.kind() == CardPositionKind::DiscardPile
    }

    /// True if this card is current in the indicated room
    pub fn is_room_occupant(&self, room_id: RoomId) -> bool {
        matches!(
            self,
            CardPosition::Room(room, location)
            if room_id == *room && *location == RoomLocation::Occupant
        )
    }

    /// Returns true if this card is in a user's score pile
    pub fn in_score_pile(&self) -> bool {
        self.kind() == CardPositionKind::Scored
    }

    /// True if this card is an identity card
    pub fn is_identity(&self) -> bool {
        self.kind() == CardPositionKind::Identity
    }
}

/// Optional card state, properties which are not universal
#[derive(PartialEq, Eq, Hash, Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardData {
    /// How many times has this card been leveled up?
    pub card_level: LevelValue,
    /// How many times the boost ability of this card has been activated --
    /// typically used to increase weapon attack power during a raid.
    pub boost_count: BoostCount,
    /// How much mana is stored in this card?
    pub stored_mana: ManaValue,
    /// When was the last time this card entered the arena, if ever?
    pub last_entered_play: Option<TurnData>,
    /// Is this card face-up?
    is_face_up: bool,
    /// Is this card revealed to the [CardId.side] user?
    revealed_to_owner: bool,
    /// Is this card revealed to opponent of the [CardId.side] user?
    revealed_to_opponent: bool,
}

/// Stores the state of a Card during an ongoing game. The game rules for a
/// card are not part of its state, see [crate::card_definition::CardDefinition]
/// for that.
#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub struct CardState {
    /// ID for this card.
    pub id: CardId,
    /// Car d name, can be used to look up this card's definition
    pub name: CardName,
    /// Optional state for this card
    pub data: CardData,
    /// Opaque value identifying this card's sort order within its CardPosition.
    /// Higher sorting keys are closer to the 'top' or 'front' of the position.
    pub sorting_key: u32,
    position: CardPosition,
}

impl CardState {
    /// Creates a new card state, placing the card into the `side` player's
    /// deck. If `is_identity` is true, the card is instead marked as revealed
    /// and placed into the player's identity zone.
    pub fn new(id: CardId, name: CardName, is_identity: bool) -> Self {
        Self {
            id,
            name,
            position: if is_identity {
                CardPosition::Identity(id.side)
            } else {
                CardPosition::DeckUnknown(id.side)
            },
            sorting_key: 0,
            data: CardData {
                revealed_to_owner: is_identity,
                revealed_to_opponent: is_identity,
                ..CardData::default()
            },
        }
    }

    pub fn side(&self) -> Side {
        self.id.side
    }

    /// Where this card is located in the game.
    pub fn position(&self) -> CardPosition {
        self.position
    }

    /// Sets the position of this card. Please use `mutations::move_card`
    /// instead of invoking this directly.
    pub fn set_position_internal(&mut self, sorting_key: u32, position: CardPosition) {
        self.sorting_key = sorting_key;
        self.position = position;
    }

    /// Whether this card is in the 'face up' state.
    pub fn is_face_up(&self) -> bool {
        self.data.is_face_up
    }

    /// Whether this card is not in the 'face up' state.
    pub fn is_face_down(&self) -> bool {
        !self.data.is_face_up
    }

    /// Change a card to the 'face up' state, but does *not* change its
    /// revealed state for either player.'
    ///
    /// Prefer `mutations::turn_face_up` to invoking this method directly.
    pub fn turn_face_up_internal(&mut self) {
        self.data.is_face_up = true;
    }

    /// Change a card to the 'face down' state, but does *not* change its
    /// revealed state for either player.
    ///
    /// Prefer `mutations::turn_face_down` to invoking this method directly.
    pub fn turn_face_down_internal(&mut self) {
        self.data.is_face_up = false;
    }

    /// Sets whether this card is revealed to the `side` player. You should
    /// typically use `mutations::set_revealed_to` instead of calling this
    /// method.
    ///
    /// Note that this is not the same as [Self::turn_face_up], both players may
    /// know a card without it being the the 'face up' state.
    pub fn set_revealed_internal(&mut self, side: Side, revealed: bool) {
        if self.id.side == side {
            self.data.revealed_to_owner = revealed
        } else {
            self.data.revealed_to_opponent = revealed
        }
    }

    /// Returns true if this card is currently revealed to the indicated user
    ///
    /// Note that this is not the same as [Self::is_face_up], both players may
    /// know a card without it being the the 'face up' state.
    pub fn is_revealed_to(&self, side: Side) -> bool {
        if self.id.side == side {
            self.data.revealed_to_owner
        } else {
            self.data.revealed_to_opponent
        }
    }
}

impl PartialOrd<Self> for CardState {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CardState {
    fn cmp(&self, other: &Self) -> Ordering {
        self.sorting_key.cmp(&other.sorting_key)
    }
}
