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
use data::game::{GameState, RaidData, RaidJumpRequest, RaidPhase, RaidState};
use data::game_actions::{GamePrompt, PromptAction, PromptContext};
use data::primitives::{RaidId, RoomId, Side};
use data::updates::{GameUpdate, InitiatedBy};
use data::with_error::WithError;
use data::{utils, verify};
use fallible_iterator::FallibleIterator;

use crate::raid::access::AccessState;
use crate::raid::activation::ActivateState;
use crate::raid::begin::BeginState;
use crate::raid::continuation::ContinueState;
use crate::raid::encounter::EncounterState;
use crate::{flags, mutations, queries};

pub trait RaidStateNode<T: Eq>: Copy {
    fn unwrap(action: PromptAction) -> Result<T>;

    fn wrap(action: T) -> Result<PromptAction>;

    fn enter(self, game: &mut GameState) -> Result<Option<RaidState>>;

    fn actions(self, game: &GameState) -> Result<Vec<T>>;

    fn active_side(self) -> Side;

    fn handle_action(self, game: &mut GameState, action: T) -> Result<Option<RaidState>>;

    fn prompt_context(self) -> Option<PromptContext> {
        None
    }

    fn handle_prompt(
        self,
        game: &mut GameState,
        action: PromptAction,
    ) -> Result<Option<RaidState>> {
        self.handle_action(game, Self::unwrap(action)?)
    }

    fn prompts(self, game: &GameState) -> Result<Vec<PromptAction>> {
        utils::fallible(self.actions(game)?.into_iter()).map(Self::wrap).collect()
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
    let state = RaidState::Begin;
    let raid = RaidData {
        target: target_room,
        raid_id,
        phase: RaidPhase::Begin,
        state,
        encounter: None,
        room_active: false,
        accessed: vec![],
        jump_request: None,
    };

    game.data.next_raid_id += 1;
    game.data.raid = Some(raid);
    on_begin(game, raid_id);
    game.record_update(|| GameUpdate::InitiateRaid(target_room, initiated_by));
    enter_state(game, Some(state))?;

    Ok(())
}

pub fn handle_action(game: &mut GameState, user_side: Side, action: PromptAction) -> Result<()> {
    let state = game.raid()?.state;
    verify!(state.active_side() == user_side, "Unexpected side");
    verify!(state.prompts(game)?.iter().any(|c| c == &action), "Unexpected action");
    let mut new_state = state.handle_action(game, action)?;
    new_state = apply_jump(game)?.or(new_state);

    if game.data.raid.is_some() {
        enter_state(game, new_state)
    } else {
        Ok(())
    }
}

pub fn current_actions(game: &GameState, user_side: Side) -> Result<Option<Vec<PromptAction>>> {
    if let Some(raid) = &game.data.raid {
        let state = raid.state;
        if state.active_side() == user_side {
            let prompts = state.prompts(game)?;
            if !prompts.is_empty() {
                return Ok(Some(prompts));
            }
        }
    }

    Ok(None)
}

pub fn current_prompt(game: &GameState, user_side: Side) -> Result<Option<GamePrompt>> {
    if let Some(actions) = current_actions(game, user_side)? {
        Ok(Some(GamePrompt { context: game.raid()?.state.prompt_context(), responses: actions }))
    } else {
        Ok(None)
    }
}

fn enter_state(game: &mut GameState, mut state: Option<RaidState>) -> Result<()> {
    loop {
        if let Some(s) = state {
            game.raid_mut()?.state = s;
            tmp_sync_state_to_phase(game)?;
            state = s.enter(game)?;
            state = apply_jump(game)?.or(state);
        } else {
            return Ok(());
        }
    }
}

fn apply_jump(game: &mut GameState) -> Result<Option<RaidState>> {
    if let Some(raid) = &game.data.raid {
        if let Some(RaidJumpRequest::EncounterMinion(card_id)) = raid.jump_request {
            let (room_id, index) =
                queries::minion_position(game, card_id).with_error(|| "Minion not found")?;
            let raid = game.raid_mut()?;
            raid.target = room_id;
            raid.encounter = Some(index);
            raid.jump_request = None;
            return Ok(Some(RaidState::Continue));
        }
    }

    Ok(None)
}

fn tmp_sync_state_to_phase(game: &mut GameState) -> Result<()> {
    match game.raid()?.state {
        RaidState::Begin => game.raid_mut()?.phase = RaidPhase::Begin,
        RaidState::Activation => game.raid_mut()?.phase = RaidPhase::Activation,
        RaidState::Encounter => {
            game.raid_mut()?.phase = RaidPhase::Encounter(game.raid_encounter()?)
        }
        RaidState::Continue => game.raid_mut()?.phase = RaidPhase::Continue(game.raid_encounter()?),
        RaidState::Access => game.raid_mut()?.phase = RaidPhase::Access,
    }

    Ok(())
}

impl RaidStateNode<PromptAction> for RaidState {
    fn unwrap(action: PromptAction) -> Result<PromptAction> {
        Ok(action)
    }

    fn wrap(action: PromptAction) -> Result<PromptAction> {
        Ok(action)
    }

    fn enter(self, game: &mut GameState) -> Result<Option<RaidState>> {
        match self {
            Self::Activation => ActivateState {}.enter(game),
            Self::Begin => BeginState {}.enter(game),
            Self::Encounter => EncounterState {}.enter(game),
            Self::Continue => ContinueState {}.enter(game),
            Self::Access => AccessState {}.enter(game),
        }
    }

    fn actions(self, game: &GameState) -> Result<Vec<PromptAction>> {
        match self {
            Self::Activation => ActivateState {}.prompts(game),
            Self::Begin => BeginState {}.prompts(game),
            Self::Encounter => EncounterState {}.prompts(game),
            Self::Continue => ContinueState {}.prompts(game),
            Self::Access => AccessState {}.prompts(game),
        }
    }

    fn active_side(self) -> Side {
        match self {
            Self::Activation => ActivateState {}.active_side(),
            Self::Begin => BeginState {}.active_side(),
            Self::Encounter => EncounterState {}.active_side(),
            Self::Continue => ContinueState {}.active_side(),
            Self::Access => AccessState {}.active_side(),
        }
    }

    fn handle_action(
        self,
        game: &mut GameState,
        action: PromptAction,
    ) -> Result<Option<RaidState>> {
        match self {
            Self::Activation => ActivateState {}.handle_prompt(game, action),
            Self::Begin => BeginState {}.handle_prompt(game, action),
            Self::Encounter => EncounterState {}.handle_prompt(game, action),
            Self::Continue => ContinueState {}.handle_prompt(game, action),
            Self::Access => AccessState {}.handle_prompt(game, action),
        }
    }

    fn prompt_context(self) -> Option<PromptContext> {
        match self {
            Self::Activation => ActivateState {}.prompt_context(),
            Self::Begin => BeginState {}.prompt_context(),
            Self::Encounter => EncounterState {}.prompt_context(),
            Self::Continue => ContinueState {}.prompt_context(),
            Self::Access => AccessState {}.prompt_context(),
        }
    }

    fn handle_prompt(
        self,
        game: &mut GameState,
        action: PromptAction,
    ) -> Result<Option<RaidState>> {
        self.handle_action(game, action)
    }

    fn prompts(self, game: &GameState) -> Result<Vec<PromptAction>> {
        self.actions(game)
    }
}
