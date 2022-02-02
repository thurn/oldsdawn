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

use anyhow::{bail, ensure, Context, Result};
use data::actions::{ActivateRoomAction, AdvanceAction, EncounterAction};
use data::card_state::CardPosition;
use data::delegates::{ChampionScoreCardEvent, MinionCombatAbilityEvent, RaidBeginEvent};
use data::game::{GameState, RaidPhase};
use data::primitives::{CardId, RoomId, Side};
use data::updates::{GameUpdate, InteractionObjectId, TargetedInteraction};
use tracing::{info, instrument};

use crate::{dispatch, flags, mutations, queries, raid_phases};

#[instrument(skip(game))]
pub fn initiate_raid_action(
    game: &mut GameState,
    user_side: Side,
    target_room: RoomId,
) -> Result<()> {
    info!(?user_side, "initiate_raid_action");
    ensure!(flags::can_initiate_raid(game, user_side), "Cannot initiate raid for {:?}", user_side);
    mutations::spend_action_points(game, user_side, 1);

    let phase = if game.defenders_alphabetical(target_room).any(|c| !c.data.revealed_to_opponent) {
        RaidPhase::Activation
    } else {
        let defender_count = game.defenders_alphabetical(target_room).count();
        if defender_count == 0 {
            RaidPhase::Access
        } else {
            RaidPhase::Encounter(defender_count - 1)
        }
    };

    let raid_id = raid_phases::create_raid(game, target_room, phase)?;
    dispatch::invoke_event(game, RaidBeginEvent(raid_id));
    game.updates.push(GameUpdate::InitiateRaid(target_room));

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

    let defender_count = game.defenders_alphabetical(game.raid()?.target).count();
    game.raid_mut()?.room_active = data == ActivateRoomAction::Activate;

    raid_phases::set_raid_phase(
        game,
        if defender_count == 0 {
            RaidPhase::Access
        } else {
            RaidPhase::Encounter(defender_count - 1)
        },
    )
}

#[instrument(skip(game))]
pub fn encounter_action(
    game: &mut GameState,
    user_side: Side,
    action: EncounterAction,
) -> Result<()> {
    info!(?user_side, ?action, "raid_encounter_action");
    ensure!(
        flags::can_take_raid_encounter_action(game, user_side, action),
        "Cannot take encounter action for {:?}",
        user_side
    );

    let encounter_number = match game.raid()?.phase {
        RaidPhase::Encounter(n) => n,
        _ => bail!("Expected Encounter phase"),
    };

    match action {
        EncounterAction::UseWeaponAbility(source_id, target_id) => {
            let cost =
                queries::cost_to_defeat_target(game, source_id, target_id).with_context(|| {
                    format!("{:?} cannot defeat target: {:?}", source_id, target_id)
                })?;
            mutations::spend_mana(game, user_side, cost);
            game.updates.push(GameUpdate::TargetedInteraction(TargetedInteraction {
                source: InteractionObjectId::CardId(source_id),
                target: InteractionObjectId::CardId(target_id),
            }))
        }
        EncounterAction::Continue => {
            let target = game.raid()?.target;
            let defender_id = raid_phases::find_defender(game, target, encounter_number)?;
            dispatch::invoke_event(game, MinionCombatAbilityEvent(defender_id));
            game.updates.push(GameUpdate::TargetedInteraction(TargetedInteraction {
                source: InteractionObjectId::CardId(defender_id),
                target: InteractionObjectId::Identity(Side::Champion),
            }));
        }
    }

    if game.data.raid.is_none() {
        // Raid may have been ended by an ability.
        Ok(())
    } else if encounter_number == 0 {
        raid_phases::set_raid_phase(game, RaidPhase::Access)
    } else {
        raid_phases::set_raid_phase(game, RaidPhase::Continue(encounter_number - 1))
    }
}

#[instrument(skip(game))]
pub fn advance_action(game: &mut GameState, user_side: Side, data: AdvanceAction) -> Result<()> {
    info!(?user_side, ?data, "raid_advance_action");
    ensure!(
        flags::can_take_raid_advance_action(game, user_side, data),
        "Cannot take advance action for {:?}",
        user_side
    );
    todo!()
}

#[instrument(skip(game))]
pub fn destroy_card_action(game: &mut GameState, user_side: Side, card_id: CardId) -> Result<()> {
    info!(?user_side, ?card_id, "raid_destroy_card_action");
    ensure!(
        flags::can_take_raid_destroy_card_action(game, user_side, card_id),
        "Cannot take destroy card action for {:?}",
        user_side
    );
    todo!()
}

#[instrument(skip(game))]
pub fn score_card_action(game: &mut GameState, user_side: Side, card_id: CardId) -> Result<()> {
    info!(?user_side, ?card_id, "raid_score_card_action");
    ensure!(
        flags::can_score_card(game, user_side, card_id),
        "Cannot take score card action for {:?}",
        user_side
    );
    let scheme_points = crate::card_definition(game, card_id)
        .config
        .stats
        .scheme_points
        .with_context(|| format!("Expected SchemePoints for {:?}", card_id))?;

    game.champion.score += scheme_points.points;
    mutations::move_card(game, card_id, CardPosition::Scored(Side::Champion));
    game.raid_mut()?.accessed.retain(|c| *c != card_id);
    raid_phases::set_raid_prompt(game)?;
    dispatch::invoke_event(game, ChampionScoreCardEvent(card_id));
    game.updates.push(GameUpdate::ChampionScoreCard(card_id, scheme_points.points));
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

    mutations::end_raid(game);
    Ok(())
}
