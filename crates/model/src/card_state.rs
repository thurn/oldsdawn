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
use crate::primitives::{
    AbilityIndex, BoostCount, CardId, ItemLocation, LevelValue, ManaValue, RoomId, RoomLocation,
    Side,
};
use std::collections::BTreeMap;
use strum_macros::EnumDiscriminants;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, EnumDiscriminants)]
#[strum_discriminants(name(CardPositionTypes))]
pub enum CardPosition {
    Room(RoomId, RoomLocation),
    ArenaItem(ItemLocation),
    Hand(Side),
    Deck(Side),
    DiscardPile(Side),
    Scored(Side),
}

impl CardPosition {
    /// Returns true if this card is in a room or has been played as an item
    pub fn in_play(&self) -> bool {
        matches!(self.into(), CardPositionTypes::Room | CardPositionTypes::ArenaItem)
    }

    pub fn in_hand(&self) -> bool {
        CardPositionTypes::Hand == self.into()
    }

    pub fn in_deck(&self) -> bool {
        CardPositionTypes::Deck == self.into()
    }

    pub fn in_discard_pile(&self) -> bool {
        CardPositionTypes::DiscardPile == self.into()
    }

    pub fn in_score_pile(&self) -> bool {
        CardPositionTypes::Scored == self.into()
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Default)]
pub struct AbilityState {}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Default)]
pub struct CardData {
    /// How many times the boost ability of this card has been activated -- typically used to
    /// increase weapon attack power during a raid.
    pub boost_count: BoostCount,
    /// How much mana is stored in this card?
    pub stored_mana: ManaValue,
    /// How many times has this card been leveled up?
    pub card_level: LevelValue,
    /// State for this card's abilities
    pub ability_state: BTreeMap<AbilityIndex, AbilityState>,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct CardState {
    id: CardId,
    name: CardName,
    position: CardPosition,
    position_modified: bool,
    data: CardData,
    data_modified: bool,
}

impl CardState {
    pub fn new(id: CardId, name: CardName, position: CardPosition) -> Self {
        Self {
            id,
            name,
            position,
            position_modified: false,
            data: CardData::default(),
            data_modified: false,
        }
    }

    /// Reset the value of 'modified' flags to false
    pub fn clear_modified_flags(&mut self) {
        self.position_modified = false;
        self.data_modified = false;
    }

    /// ID for this card.
    pub fn id(&self) -> CardId {
        self.id
    }

    /// Card name, can be used to look up this card's [CardDefinition]
    pub fn name(&self) -> CardName {
        self.name
    }

    /// Where this card is located in the game
    pub fn position(&self) -> CardPosition {
        self.position
    }

    /// Move this card to a new position
    pub fn move_to(&mut self, new_position: CardPosition) {
        self.position_modified = true;
        self.position = new_position;
    }

    /// Whether [Self::position] has been modified since the last call to
    /// [CardState::clear_modified_flags]    
    pub fn position_modified(&self) -> bool {
        self.position_modified
    }

    /// Optional state for this card
    pub fn data(&self) -> &CardData {
        &self.data
    }

    /// Mutable version of [Self::data]
    pub fn data_mut(&mut self) -> &mut CardData {
        self.data_modified = true;
        &mut self.data
    }

    /// Whether [Self::data] has been modified since the last call to
    /// [CardState::clear_modified_flags]
    pub fn data_modified(&self) -> bool {
        self.data_modified
    }
}
