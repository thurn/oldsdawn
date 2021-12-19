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

use crate::card_definition::CardDefinition;
use crate::card_name::CardName;
use crate::deck::Deck;
use crate::game::GameState;
use crate::primitives::{
    AbilityIndex, BoostCount, CardId, ItemLocation, LevelValue, ManaValue, RoomId, RoomLocation,
    Side,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use strum_macros::EnumDiscriminants;

/// Determines display order when multiple cards are in the same position. Typically, this is taken
/// from an opaque, sequentially increasing counter representing what time the card first moved to
/// this position.
pub type SortingKey = u32;

/// Possible known positions of cards within a deck
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum DeckPosition {
    Unknown,
    Top,
    Bottom,
}

/// Identifies the location of a card during an active game
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, EnumDiscriminants, Serialize, Deserialize)]
#[strum_discriminants(name(CardPositionTypes))]
pub enum CardPosition {
    /// An unspecified random position within a user's deck. The default position of all cards
    /// when a new game is started.
    DeckUnknown(Side),
    /// A position within a user's deck which is known to at least one player.
    DeckKnown(Side, DeckPosition),
    Hand(Side),
    Room(RoomId, RoomLocation),
    ArenaItem(ItemLocation),
    DiscardPile(Side),
    Scored(Side),
    /// Marks the identity card for a side. It is an error for a game to contain more than one
    /// identity card per side.
    Identity(Side),
}

impl CardPosition {
    /// Returns true if this card is in a room or has been played as an item
    pub fn in_play(&self) -> bool {
        matches!(self.into(), CardPositionTypes::Room | CardPositionTypes::ArenaItem)
    }

    pub fn in_hand(&self) -> bool {
        CardPositionTypes::Hand == self.into()
    }

    /// Returns true if this card is in a known or unknown deck position
    pub fn in_deck(&self) -> bool {
        matches!(self.into(), CardPositionTypes::DeckUnknown | CardPositionTypes::DeckKnown)
    }

    pub fn in_discard_pile(&self) -> bool {
        CardPositionTypes::DiscardPile == self.into()
    }

    pub fn in_score_pile(&self) -> bool {
        CardPositionTypes::Scored == self.into()
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Default, Serialize, Deserialize)]
pub struct AbilityState {}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Default, Serialize, Deserialize)]
pub struct CardData {
    // Has this card been revealed to the opponent?
    pub revealed: bool,
    /// How many times has this card been leveled up?
    pub card_level: LevelValue,
    /// How many times the boost ability of this card has been activated -- typically used to
    /// increase weapon attack power during a raid.
    pub boost_count: BoostCount,
    /// How much mana is stored in this card?
    pub stored_mana: ManaValue,
    /// State for this card's abilities
    pub ability_state: BTreeMap<AbilityIndex, AbilityState>,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub struct CardState {
    /// ID for this card.
    pub id: CardId,
    /// Card name, can be used to look up this card's [CardDefinition]
    pub name: CardName,
    /// Player who owns this card
    pub side: Side,
    /// Where this card is located in the game. Use [GameState::move_card] instead of modifying
    /// this directly.
    pub position: CardPosition,
    /// Opaque value identifying this card's sort order within its position
    pub sorting_key: SortingKey,
    /// Optional state for this card
    pub data: CardData,
}

impl CardState {
    pub fn new(id: CardId, name: CardName, side: Side) -> Self {
        Self {
            id,
            name,
            side,
            position: CardPosition::DeckUnknown(side),
            sorting_key: 0,
            data: CardData::default(),
        }
    }
}
