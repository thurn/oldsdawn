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
use std::collections::BTreeMap;

use enum_kinds::EnumKind;
use serde::{Deserialize, Serialize};

use crate::card_name::CardName;
use crate::primitives::{
    AbilityIndex, BoostCount, CardId, ItemLocation, LevelValue, ManaValue, RoomId, RoomLocation,
    Side,
};

/// Determines display order when multiple cards are in the same position.
/// Typically, this is taken from an opaque, sequentially increasing counter
/// representing what time the card first moved to this position.
pub type SortingKey = u32;

/// Identifies the location of a card during an active game
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, EnumKind, Serialize, Deserialize)]
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
    /// Marks the identity card for a side. It is an error for a game to contain
    /// more than one identity card per side.
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

    /// Returns true if this card is in a known or unknown deck position
    pub fn in_deck(&self) -> bool {
        matches!(self.kind(), CardPositionKind::DeckUnknown | CardPositionKind::DeckTop)
    }

    /// Returns true if this card is in a user's discard pile
    pub fn in_discard_pile(&self) -> bool {
        self.kind() == CardPositionKind::DiscardPile
    }

    /// Returns true if this card is in a user's score pile
    pub fn in_score_pile(&self) -> bool {
        self.kind() == CardPositionKind::Scored
    }
}

/// Stores the state of a specific ability of a card
#[derive(PartialEq, Eq, Hash, Debug, Clone, Default, Serialize, Deserialize)]
pub struct AbilityState {}

/// Optional card state, properties which are not universal
#[derive(PartialEq, Eq, Hash, Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardData {
    // Has this card been revealed to the opponent?
    pub revealed: bool,
    /// How many times has this card been leveled up?
    pub card_level: LevelValue,
    /// How many times the boost ability of this card has been activated --
    /// typically used to increase weapon attack power during a raid.
    pub boost_count: BoostCount,
    /// How much mana is stored in this card?
    pub stored_mana: ManaValue,
    /// State for this card's abilities
    pub ability_state: BTreeMap<AbilityIndex, AbilityState>,
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
    /// Player who owns this card
    pub side: Side,
    /// Optional state for this card
    pub data: CardData,
    /// Where this card is located in the game. Use
    /// [crate::game::GameState::move_card] instead of modifying this
    /// directly.
    pub position: CardPosition,
    /// Opaque value identifying this card's sort order within its position
    pub sorting_key: SortingKey,
}

impl CardState {
    /// Creates a new card state, placing the card into the `side` player's
    /// deck. If `is_identity` is true, the card is instead marked as revealed
    /// and placed into the player's identity zone.
    pub fn new(id: CardId, name: CardName, side: Side, is_identity: bool) -> Self {
        Self {
            id,
            name,
            side,
            position: if is_identity {
                CardPosition::Identity(side)
            } else {
                CardPosition::DeckUnknown(side)
            },
            sorting_key: 0,
            data: CardData { revealed: is_identity, ..CardData::default() },
        }
    }

    /// Returns true if this card is currently revealed to the indicated user
    pub fn is_revealed_to(&self, side: Side) -> bool {
        if self.position.kind() == CardPositionKind::DeckUnknown {
            false
        } else if self.id.side == side {
            true
        } else {
            self.data.revealed
        }
    }

    pub fn is_in_room(&self, room_id: RoomId) -> bool {
        matches!(self.position, CardPosition::Room(id, _) if id == room_id)
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
