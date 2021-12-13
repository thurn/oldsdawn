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

use crate::primitives::{
    AbilityIndex, CardId, EncounterId, ItemLocation, RoomId, RoomLocation, Side, TurnNumber,
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

/// Stores the last activation turn & encounter for an ability. This value is automatically updated
/// by the system when an ability is activated, immediately before the ON_ABILITY_ACTIVATED event
/// is sent.
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct LastActivated {
    pub turn_number: TurnNumber,
    pub encounter_id: EncounterId,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Default)]
pub struct AbilityState {
    pub last_activated: Option<LastActivated>,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct CardState {
    pub id: CardId,
    pub position: CardPosition,
    pub state: BTreeMap<AbilityIndex, AbilityState>,
}
