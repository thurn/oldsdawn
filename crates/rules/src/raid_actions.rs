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

use anyhow::{bail, ensure, Error, Result};
use data::card_state::{CardPosition, CardState};
use data::delegates::{
    ChampionScoreCardEvent, MinionCombatAbilityEvent, MinionDefeatedEvent, RaidOutcome, RaidStart,
    RaidStartEvent, ScoreCard, ScoreCardEvent, UsedWeapon, UsedWeaponEvent,
};
use data::game::{GameState, RaidData, RaidPhase};
use data::game_actions::{ContinueAction, EncounterAction, RoomActivationAction};
use data::primitives::{CardId, GameObjectId, RaidId, RoomId, Side};
use data::updates::{GameUpdate, TargetedInteraction};
use data::with_error::WithError;
use data::{fail, utils, verify};
use fallible_iterator::{Convert, FallibleIterator, IntoFallibleIterator};
use tracing::{info, instrument};

use crate::card_prompt::HandleCardPrompt;
use crate::mana::ManaPurpose;
use crate::{card_prompt, dispatch, flags, mana, mutations, queries, raid_phases};

#[instrument(skip(game))]
pub fn initiate_raid_action(
    game: &mut GameState,
    user_side: Side,
    target_room: RoomId,
) -> Result<()> {
    info!(?user_side, "initiate_raid_action");
    verify!(
        flags::can_take_initiate_raid_action(game, user_side, target_room),
        "Cannot initiate raid for {:?}",
        user_side
    );
    mutations::spend_action_points(game, user_side, 1)?;
    initiate_raid(game, target_room, |_, _| {})?;
    Ok(())
}

/// Initiates a raid on the indicated `target_room`.
///
/// The `on_begin` function is invoked with the [RaidId] once the raid is
/// created, with the raid phase set to [RaidPhase::Begin].
pub fn initiate_raid(
    game: &mut GameState,
    target_room: RoomId,
    on_begin: impl Fn(&mut GameState, RaidId),
) -> Result<()> {
    let raid_id = RaidId(game.data.next_raid_id);
    let raid = RaidData {
        target: target_room,
        raid_id,
        phase: RaidPhase::Begin,
        room_active: false,
        accessed: vec![],
    };

    game.data.next_raid_id += 1;
    game.data.raid = Some(raid);
    on_begin(game, raid_id);
    game.record_update(|| GameUpdate::InitiateRaid(target_room));

    let phase = if game.defenders_unordered(target_room).any(CardState::is_face_down) {
        RaidPhase::Activation
    } else {
        next_encounter(game, None, RaidPhase::Encounter)?
    };

    raid_phases::set_raid_phase(game, phase)?;
    dispatch::invoke_event(game, RaidStartEvent(RaidStart { raid_id, target: target_room }))?;
    Ok(())
}

#[instrument(skip(game))]
pub fn room_activation_action(
    game: &mut GameState,
    user_side: Side,
    data: RoomActivationAction,
) -> Result<()> {
    info!(?user_side, ?data, "raid_activate_room_action");
    verify!(
        flags::can_take_room_activation_action(game, user_side),
        "Cannot activate room for {:?}",
        user_side
    );

    game.raid_mut()?.room_active = data == RoomActivationAction::Activate;
    raid_phases::set_raid_phase(game, next_encounter(game, None, RaidPhase::Encounter)?)
}

#[instrument(skip(game))]
pub fn encounter_action(
    game: &mut GameState,
    user_side: Side,
    action: EncounterAction,
) -> Result<()> {
    info!(?user_side, ?action, "raid_encounter_action");
    verify!(
        flags::can_take_raid_encounter_action(game, user_side, action),
        "Cannot take encounter action for {:?}",
        user_side
    );

    let mut encounter_number = get_encounter_number(game)?;
    match action {
        EncounterAction::UseWeaponAbility(source_id, target_id) => {
            let cost = queries::cost_to_defeat_target(game, source_id, target_id)
                .with_error(|| format!("{:?} cannot defeat target: {:?}", source_id, target_id))?;
            mana::spend(game, user_side, ManaPurpose::UseWeapon(source_id), cost)?;

            game.record_update(|| {
                GameUpdate::TargetedInteraction(TargetedInteraction {
                    source: GameObjectId::CardId(source_id),
                    target: GameObjectId::CardId(target_id),
                })
            });

            dispatch::invoke_event(
                game,
                UsedWeaponEvent(UsedWeapon {
                    raid_id: game.raid()?.raid_id,
                    weapon_id: source_id,
                    target_id,
                    mana_spent: cost,
                }),
            )?;
            dispatch::invoke_event(game, MinionDefeatedEvent(target_id))?;
        }
        EncounterAction::NoWeapon | EncounterAction::CardAction(_) => {
            let target = game.raid()?.target;
            let defender_id = raid_phases::find_defender(game, target, encounter_number)?;
            game.record_update(|| {
                GameUpdate::TargetedInteraction(TargetedInteraction {
                    source: GameObjectId::CardId(defender_id),
                    target: GameObjectId::Identity(Side::Champion),
                })
            });
            dispatch::invoke_event(game, MinionCombatAbilityEvent(defender_id))?;
        }
    }

    if let EncounterAction::CardAction(card_action) = action {
        card_prompt::handle(game, user_side, card_action, HandleCardPrompt::NoAction)?;
    }

    if game.data.raid.is_none() {
        // Raid may have been ended by an ability.
        Ok(())
    } else {
        // Minion abilities may change the current encounter position
        encounter_number = get_encounter_number(game)?;
        raid_phases::set_raid_phase(
            game,
            next_encounter(game, Some(encounter_number), RaidPhase::Continue)?,
        )
    }
}

/// Returns the current encounter index for a raid in the 'Encounter' phase.
fn get_encounter_number(game: &GameState) -> Result<usize> {
    match game.raid()?.phase {
        RaidPhase::Encounter(n) => Ok(n),
        _ => fail!("Expected Encounter phase"),
    }
}

#[instrument(skip(game))]
pub fn continue_action(
    game: &mut GameState,
    user_side: Side,
    action: ContinueAction,
) -> Result<()> {
    info!(?user_side, ?action, "raid_advance_action");
    verify!(
        flags::can_take_continue_action(game, user_side),
        "Cannot take advance action for {:?}",
        user_side
    );
    let encounter_number = match game.raid()?.phase {
        RaidPhase::Continue(n) => n,
        _ => fail!("Expected Continue phase"),
    };

    match action {
        ContinueAction::Advance => {
            raid_phases::set_raid_phase(game, RaidPhase::Encounter(encounter_number))
        }
        ContinueAction::Retreat => {
            mutations::end_raid(game, RaidOutcome::Failure)?;
            Ok(())
        }
    }
}

#[instrument(skip(game))]
pub fn destroy_card_action(game: &mut GameState, user_side: Side, card_id: CardId) -> Result<()> {
    info!(?user_side, ?card_id, "raid_destroy_card_action");
    verify!(
        flags::can_take_raid_destroy_card_action(game, user_side, card_id),
        "Cannot take destroy card action for {:?}",
        user_side
    );
    todo!()
}

#[instrument(skip(game))]
pub fn score_card_action(game: &mut GameState, user_side: Side, card_id: CardId) -> Result<()> {
    info!(?user_side, ?card_id, "raid_score_card_action");
    verify!(
        flags::can_score_during_raid(game, user_side, card_id),
        "Cannot take score card action for {:?}",
        user_side
    );
    let scheme_points = crate::card_definition(game, card_id)
        .config
        .stats
        .scheme_points
        .with_error(|| format!("Expected SchemePoints for {:?}", card_id))?;

    game.card_mut(card_id).turn_face_up();
    mutations::move_card(game, card_id, CardPosition::Scoring)?;
    game.raid_mut()?.accessed.retain(|c| *c != card_id);
    raid_phases::set_raid_prompt(game)?;
    game.record_update(|| GameUpdate::ScoreCard(Side::Champion, card_id));

    dispatch::invoke_event(game, ChampionScoreCardEvent(card_id))?;
    dispatch::invoke_event(game, ScoreCardEvent(ScoreCard { player: Side::Champion, card_id }))?;
    mutations::score_points(game, Side::Champion, scheme_points.points)?;
    mutations::move_card(game, card_id, CardPosition::Scored(Side::Champion))?;
    Ok(())
}

#[instrument(skip(game))]
pub fn raid_end_action(game: &mut GameState, user_side: Side) -> Result<()> {
    info!(?user_side, "raid_end_action");
    verify!(
        flags::can_take_raid_end_action(game, user_side),
        "Cannot take raid end action for {:?}",
        user_side
    );

    mutations::end_raid(game, RaidOutcome::Success)?;
    Ok(())
}

/// Searches for the next defender to encounter during an ongoing raid with a
/// position less than the provided index  (or any index if not provided). If an
/// eligible defender is available with position < `less_than`, invokes
/// `constructor` with that position. Otherwise, returns `RaidPhase::Access`.
///
/// An 'eligible' defender is either one which is face up, or one which *can* be
/// turned face up by paying its costs
/// [RaidData::room_active] is true.
fn next_encounter(
    game: &GameState,
    less_than: Option<usize>,
    constructor: impl Fn(usize) -> RaidPhase,
) -> Result<RaidPhase> {
    let defenders = game.defender_list(game.raid()?.target);
    let mut reversed = utils::fallible(defenders.iter().enumerate().rev());
    let position = reversed.find(|(index, card_id)| {
        let in_range = less_than.map_or(true, |less_than| *index < less_than);
        let can_encounter =
            game.card(**card_id).is_face_up() || raid_phases::can_summon_defender(game, *index)?;
        Ok(in_range && can_encounter)
    })?;

    Ok(if let Some((index, _)) = position { constructor(index) } else { RaidPhase::Access })
}
