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
    AbilityIndex, BoostCount, CardId, ItemLocation, RoomId, RoomLocation, Side,
};
use std::collections::BTreeMap;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum CardPosition {
    Room(RoomId, RoomLocation),
    ArenaItem(ItemLocation),
    Hand(Side),
    Deck(Side),
    DiscardPile(Side),
    Scored(Side),
}

impl CardPosition {
    /// Returns true if this position is an arena position
    pub fn in_play(&self) -> bool {
        matches!(self, Self::Room(_, _) | Self::ArenaItem(_))
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Default)]
pub struct AbilityState {}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct CardState {
    /// ID for this card.
    id: CardId,
    pub name: CardName,
    /// Where this card is located in the game
    pub position: CardPosition,
    /// State for this card's abilities
    pub state: BTreeMap<AbilityIndex, AbilityState>,
    /// How many times the boost ability of this card has been activated -- typically used to
    /// increase weapon attack power during a raid.
    pub boost_count: BoostCount,
}

impl CardState {
    pub fn new(id: CardId, name: CardName, position: CardPosition) -> Self {
        Self { id, name, state: BTreeMap::new(), position, boost_count: 0 }
    }

    pub fn id(&self) -> CardId {
        self.id
    }
}
