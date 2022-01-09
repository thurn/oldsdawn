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

//! Handling for raid-related user actions.

use anyhow::{ensure, Result};
use data::game::GameState;
use data::primitives::{CardId, RoomId, Side};
use data::prompt::{RaidActivateRoom, RaidAdvance, RaidEncounter};
use tracing::{info, instrument};

use crate::{flags, mutations};

#[instrument(skip(game))]
pub fn initiate_raid_action(
    game: &mut GameState,
    user_side: Side,
    target_room: RoomId,
) -> Result<()> {
    info!(?user_side, "initiate_raid_action");
    ensure!(flags::can_initiate_raid(game, user_side), "Cannot initiate raid for {:?}", user_side);
    mutations::spend_action_points(game, user_side, 1);
    mutations::initiate_raid(game, target_room);
    Ok(())
}

#[instrument(skip(game))]
pub fn activate_room_action(
    game: &mut GameState,
    user_side: Side,
    data: RaidActivateRoom,
) -> Result<()> {
    info!(?user_side, ?data, "raid_activate_room_action");
    ensure!(
        flags::can_take_raid_activate_room_action(game, user_side, data),
        "Cannot activate room for {:?}",
        user_side
    );
    Ok(())
}

#[instrument(skip(game))]
pub fn encounter_action(game: &mut GameState, user_side: Side, data: RaidEncounter) -> Result<()> {
    info!(?user_side, ?data, "raid_encounter_action");
    ensure!(
        flags::can_take_raid_encounter_action(game, user_side, data),
        "Cannot take encounter action for {:?}",
        user_side
    );
    Ok(())
}

#[instrument(skip(game))]
pub fn advance_action(game: &mut GameState, user_side: Side, data: RaidAdvance) -> Result<()> {
    info!(?user_side, ?data, "raid_advance_action");
    ensure!(
        flags::can_take_raid_advance_action(game, user_side, data),
        "Cannot take advance action for {:?}",
        user_side
    );
    Ok(())
}

#[instrument(skip(game))]
pub fn destroy_card_action(game: &mut GameState, user_side: Side, card_id: CardId) -> Result<()> {
    info!(?user_side, ?card_id, "raid_destroy_card_action");
    ensure!(
        flags::can_take_raid_destroy_card_action(game, user_side, card_id),
        "Cannot take destroy card action for {:?}",
        user_side
    );
    Ok(())
}

#[instrument(skip(game))]
pub fn score_card_action(game: &mut GameState, user_side: Side, card_id: CardId) -> Result<()> {
    info!(?user_side, ?card_id, "raid_score_card_action");
    ensure!(
        flags::can_take_raid_score_card_action(game, user_side, card_id),
        "Cannot take score card action for {:?}",
        user_side
    );
    Ok(())
}

#[instrument(skip(game))]
pub fn raid_end_action(game: &mut GameState, user_side: Side) -> Result<()> {
    info!(?user_side, "raid_end_action");
    ensure!(
        flags::can_take_raid_end_action(game, user_side),
        "Cannot take raid end action for {:?}",
        user_side
    );
    Ok(())
}
