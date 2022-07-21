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
use data::delegates::{RaidStart, RaidStartEvent};
use data::game::{GameState, InternalRaidPhase};
use data::game_actions::PromptAction;
use data::primitives::Side;
use rules::dispatch;
use with_error::fail;

use crate::defenders;
use crate::traits::{RaidDisplayState, RaidPhaseImpl};

/// The starting state for all raids, fires 'raid start' events and determines
/// the next state to move to.
#[derive(Debug, Clone, Copy)]
pub struct BeginPhase {}

impl RaidPhaseImpl for BeginPhase {
    type Action = ();

    fn unwrap(_: PromptAction) -> Result<()> {
        fail!("No actions for Begin Phase")
    }

    fn wrap(_: ()) -> Result<PromptAction> {
        fail!("No actions for Begin Phase")
    }

    fn enter(self, game: &mut GameState) -> Result<Option<InternalRaidPhase>> {
        dispatch::invoke_event(
            game,
            RaidStartEvent(RaidStart {
                raid_id: game.raid()?.raid_id,
                target: game.raid()?.target,
            }),
        )?;

        if game.data.raid.is_none() {
            return Ok(None);
        }

        Ok(Some(if let Some(encounter) = defenders::next_encounter(game, None)? {
            game.raid_mut()?.encounter = Some(encounter);
            InternalRaidPhase::Encounter
        } else {
            InternalRaidPhase::Access
        }))
    }

    fn actions(self, _: &GameState) -> Result<Vec<()>> {
        Ok(vec![])
    }

    fn handle_action(self, _: &mut GameState, _: ()) -> Result<Option<InternalRaidPhase>> {
        fail!("No actions for Begin Phase")
    }

    fn active_side(self) -> Side {
        Side::Champion
    }

    fn display_state(self, _: &GameState) -> Result<RaidDisplayState> {
        Ok(RaidDisplayState::None)
    }
}
