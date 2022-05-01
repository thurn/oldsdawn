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

use data::game::GameState;
use data::primitives::{AbilityId, CardId, ManaValue, RoomId, Side};

/// Identifies possible reasons why a player's mana value would need to be
/// queried or spent.
#[derive(Debug, Clone, Copy)]
pub enum ManaType {
    BaseForDisplay,
    BonusForDisplay,
    PayForCard(CardId),
    UseWeaponAbility(CardId),
    ActivateAbility(AbilityId),
    LevelUpRoom(RoomId),
    AllSources,
}

pub fn get(game: &GameState, side: Side, _mana_type: ManaType) -> ManaValue {
    game.player(side).mana_state.base_mana
}

pub fn spend(game: &mut GameState, side: Side, mana_type: ManaType, amount: ManaValue) {
    assert!(get(game, side, mana_type) >= amount);
    game.player_mut(side).mana_state.base_mana -= amount;
}

pub fn gain(game: &mut GameState, side: Side, amount: ManaValue) {
    game.player_mut(side).mana_state.base_mana += amount
}

pub fn set(game: &mut GameState, side: Side, amount: ManaValue) {
    game.player_mut(side).mana_state.base_mana = amount;
}
