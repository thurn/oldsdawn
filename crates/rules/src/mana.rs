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

//! Manages mana, especially logic around "spend this mana only on X"
//! restrictions.

use std::cmp;

use anyhow::Result;
use data::game::{GameState, SpecificRaidMana};
use data::primitives::{AbilityId, CardId, ManaValue, RaidId, RoomId, Side};
use with_error::{verify, WithError};

/// Identifies possible reasons why a player's mana value would need to be
/// queried or spent.
#[derive(Debug, Clone, Copy)]
pub enum ManaPurpose {
    BaseMana,
    BonusForDisplay,
    PayForCard(CardId),
    DestroyCard(CardId),
    UseWeapon(CardId),
    ActivateAbility(AbilityId),
    LevelUpRoom(RoomId),
    PayForTriggeredAbility,
    AllSources,
}

/// Queries the amount of mana available for the `side` player when used for the
/// given [ManaPurpose].
///
/// Certain card effects may grant mana conditionally for a given purpose.
pub fn get(game: &GameState, side: Side, purpose: ManaPurpose) -> ManaValue {
    let base_mana = game.player(side).mana_state.base_mana;
    let mut result = game.player(side).mana_state.base_mana;
    match (&game.data.raid, &game.player(side).mana_state.specific_raid_mana) {
        (Some(raid_data), Some(raid_mana)) if raid_data.raid_id == raid_mana.raid_id => {
            result += raid_mana.mana;
        }
        _ => {}
    }

    match purpose {
        ManaPurpose::BaseMana => base_mana,
        ManaPurpose::BonusForDisplay => result - base_mana,
        _ => result,
    }
}

/// Spends mana for the `side` player for the given [ManaPurpose].
///
/// An effort is made to spend "more specific" mana first, i.e. mana which can
/// only be used for a certain type of action is preferred, then raid-specific
/// mana, then general mana.
///
/// Returns an error if insufficient mana is available.
pub fn spend(
    game: &mut GameState,
    side: Side,
    purpose: ManaPurpose,
    amount: ManaValue,
) -> Result<()> {
    verify!(get(game, side, purpose) >= amount);
    let mut to_spend = amount;

    match (&game.data.raid, &game.player(side).mana_state.specific_raid_mana) {
        (Some(raid_data), Some(raid_mana)) if raid_data.raid_id == raid_mana.raid_id => {
            to_spend = try_spend(
                &mut game
                    .player_mut(side)
                    .mana_state
                    .specific_raid_mana
                    .as_mut()
                    .with_error(|| "Expected raid-specific mana")?
                    .mana,
                to_spend,
            );
        }
        _ => {}
    }

    game.player_mut(side).mana_state.base_mana -= to_spend;
    Ok(())
}

/// Causes a player to lose up to a given amount of mana.
pub fn lose_upto(game: &mut GameState, side: Side, purpose: ManaPurpose, amount: ManaValue) {
    spend(game, side, purpose, cmp::min(get(game, side, purpose), amount))
        .expect("Error spending mana");
}

/// Adds the specified amount of base mana (no restrictions on use) for the
/// `side` player.
pub fn gain(game: &mut GameState, side: Side, amount: ManaValue) {
    game.player_mut(side).mana_state.base_mana += amount
}

/// Sets an amount of base mana for the `side` player.
pub fn set(game: &mut GameState, side: Side, amount: ManaValue) {
    game.player_mut(side).mana_state.base_mana = amount;
}

/// Adds mana for the `side` player which can only be used during the specified
/// `raid_id` raid.
pub fn add_raid_specific_mana(
    game: &mut GameState,
    side: Side,
    raid_id: RaidId,
    amount: ManaValue,
) {
    match &mut game.player_mut(side).mana_state.specific_raid_mana {
        Some(raid_mana) if raid_mana.raid_id == raid_id => raid_mana.mana += amount,
        _ => {
            game.player_mut(side).mana_state.specific_raid_mana =
                Some(SpecificRaidMana { raid_id, mana: amount });
        }
    }
}

fn try_spend(source: &mut ManaValue, amount: ManaValue) -> ManaValue {
    if *source >= amount {
        *source -= amount;
        0
    } else {
        let result = amount - *source;
        *source = 0;
        result
    }
}
