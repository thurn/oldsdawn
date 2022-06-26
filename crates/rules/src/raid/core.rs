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

use anyhow::Result;
use data::game::{GameState, InternalRaidPhase, RaidData, RaidJumpRequest};
use data::game_actions::{GamePrompt, PromptAction};
use data::primitives::{RaidId, RoomId, Side};
use data::updates::{GameUpdate, InitiatedBy};
use data::verify;
use data::with_error::WithError;

use crate::raid::access::AccessPhase;
use crate::raid::activation::ActivationPhase;
use crate::raid::begin::BeginPhase;
use crate::raid::continuation::ContinuePhase;
use crate::raid::encounter::EncounterPhase;
use crate::raid::traits::RaidPhase;
use crate::{flags, mutations, queries};

pub trait RaidDataExt {
    fn phase(&self) -> Box<dyn RaidPhase>;
}

impl RaidDataExt for RaidData {
    fn phase(&self) -> Box<dyn RaidPhase> {
        match self.internal_phase {
            InternalRaidPhase::Begin => Box::new(BeginPhase {}),
            InternalRaidPhase::Activation => Box::new(ActivationPhase {}),
            InternalRaidPhase::Encounter => Box::new(EncounterPhase {}),
            InternalRaidPhase::Continue => Box::new(ContinuePhase {}),
            InternalRaidPhase::Access => Box::new(AccessPhase {}),
        }
    }
}

pub fn handle_initiate_action(
    game: &mut GameState,
    user_side: Side,
    target_room: RoomId,
) -> Result<()> {
    verify!(
        flags::can_take_initiate_raid_action(game, user_side, target_room),
        "Cannot initiate raid for {:?}",
        user_side
    );
    mutations::spend_action_points(game, user_side, 1)?;
    initiate(game, target_room, InitiatedBy::GameAction, |_, _| {})
}

pub fn initiate(
    game: &mut GameState,
    target_room: RoomId,
    initiated_by: InitiatedBy,
    on_begin: impl Fn(&mut GameState, RaidId),
) -> Result<()> {
    let raid_id = RaidId(game.data.next_raid_id);
    let phase = InternalRaidPhase::Begin;
    let raid = RaidData {
        target: target_room,
        raid_id,
        internal_phase: phase,
        encounter: None,
        room_active: false,
        accessed: vec![],
        jump_request: None,
    };

    game.data.next_raid_id += 1;
    game.data.raid = Some(raid);
    on_begin(game, raid_id);
    game.record_update(|| GameUpdate::InitiateRaid(target_room, initiated_by));
    enter_phase(game, Some(phase))?;

    Ok(())
}

pub fn handle_action(game: &mut GameState, user_side: Side, action: PromptAction) -> Result<()> {
    let phase = game.raid()?.phase();
    verify!(phase.active_side() == user_side, "Unexpected side");
    verify!(phase.prompts(game)?.iter().any(|c| c == &action), "Unexpected action");
    let mut new_state = phase.handle_prompt(game, action)?;
    new_state = apply_jump(game)?.or(new_state);

    if game.data.raid.is_some() {
        enter_phase(game, new_state)
    } else {
        Ok(())
    }
}

pub fn current_actions(game: &GameState, user_side: Side) -> Result<Option<Vec<PromptAction>>> {
    if let Some(raid) = &game.data.raid {
        if raid.phase().active_side() == user_side {
            let prompts = raid.phase().prompts(game)?;
            if !prompts.is_empty() {
                return Ok(Some(prompts));
            }
        }
    }

    Ok(None)
}

pub fn current_prompt(game: &GameState, user_side: Side) -> Result<Option<GamePrompt>> {
    if let Some(actions) = current_actions(game, user_side)? {
        Ok(Some(GamePrompt { context: game.raid()?.phase().prompt_context(), responses: actions }))
    } else {
        Ok(None)
    }
}

fn enter_phase(game: &mut GameState, mut phase: Option<InternalRaidPhase>) -> Result<()> {
    loop {
        if let Some(s) = phase {
            game.raid_mut()?.internal_phase = s;
            phase = game.raid()?.phase().enter(game)?;
            phase = apply_jump(game)?.or(phase);
        } else {
            return Ok(());
        }
    }
}

fn apply_jump(game: &mut GameState) -> Result<Option<InternalRaidPhase>> {
    if let Some(raid) = &game.data.raid {
        if let Some(RaidJumpRequest::EncounterMinion(card_id)) = raid.jump_request {
            let (room_id, index) =
                queries::minion_position(game, card_id).with_error(|| "Minion not found")?;
            let raid = game.raid_mut()?;
            raid.target = room_id;
            raid.encounter = Some(index);
            raid.jump_request = None;
            return Ok(Some(InternalRaidPhase::Continue));
        }
    }

    Ok(None)
}
