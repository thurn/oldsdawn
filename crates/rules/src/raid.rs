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

use std::iter;

use anyhow::{ensure, Context, Result};
use data::actions::{
    ActivateRoomAction, AdvanceAction, EncounterAction, Prompt, PromptAction, PromptKind,
};
use data::game::{GameState, RaidPhase};
use data::primitives::{CardId, RoomId, Side};
use if_chain::if_chain;
use tracing::{info, instrument};

use crate::{flags, mutations, queries};

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
    data: ActivateRoomAction,
) -> Result<()> {
    info!(?user_side, ?data, "raid_activate_room_action");
    ensure!(
        flags::can_take_raid_activate_room_action(game, user_side),
        "Cannot activate room for {:?}",
        user_side
    );

    let defender_count = game
        .defenders_alphabetical(game.data.raid.with_context(|| "No active raid")?.target)
        .count();
    let raid = game.data.raid.as_mut().with_context(|| "No active raid")?;
    raid.active = data == ActivateRoomAction::Activate;

    if defender_count == 0 {
        // TODO: Access cards
        return Ok(());
    }

    raid.phase = RaidPhase::Encounter(defender_count - 1);
    let target = raid.target;
    let defender_id =
        game.defender_list(target).get(defender_count - 1).with_context(|| "No defender")?.id;

    if_chain! {
        if let Some(cost) = queries::mana_cost(game, defender_id);
        if cost <= game.player(Side::Overlord).mana;
        then {
            mutations::spend_mana(game, Side::Overlord, cost);
            mutations::set_revealed(game, defender_id, true);

            mutations::set_prompt(
                game,
                Side::Champion,
                Prompt {
                    kind: PromptKind::EncounterAction,
                    responses: game
                        .weapons()
                        .filter(|weapon| flags::can_defeat_target(game, weapon.id, defender_id))
                        .map(|weapon| {
                            PromptAction::EncounterAction(EncounterAction::UseWeaponAbility(
                                weapon.id,
                                defender_id,
                            ))
                        })
                        .chain(iter::once(PromptAction::EncounterAction(
                            EncounterAction::Continue,
                        )))
                        .collect(),
                },
            );
        } else {
            // TODO: Continue
        }
    }

    Ok(())
}

#[instrument(skip(game))]
pub fn encounter_action(
    game: &mut GameState,
    user_side: Side,
    data: EncounterAction,
) -> Result<()> {
    info!(?user_side, ?data, "raid_encounter_action");
    ensure!(
        flags::can_take_raid_encounter_action(game, user_side, data),
        "Cannot take encounter action for {:?}",
        user_side
    );
    Ok(())
}

#[instrument(skip(game))]
pub fn advance_action(game: &mut GameState, user_side: Side, data: AdvanceAction) -> Result<()> {
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
