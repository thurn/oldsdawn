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
use data::delegates::RaidOutcome;
use data::fail;
use data::game::{GameState, RaidState};
use data::game_actions::{ContinueAction, PromptAction, PromptContext};
use data::primitives::Side;

use crate::mutations;
use crate::raid::core::{RaidDisplayState, RaidStateNode};

#[derive(Debug, Clone, Copy)]
pub struct ContinueState {}

impl RaidStateNode<ContinueAction> for ContinueState {
    fn unwrap(action: PromptAction) -> Result<ContinueAction> {
        match action {
            PromptAction::ContinueAction(action) => Ok(action),
            _ => fail!("Expected ContinueAction"),
        }
    }

    fn wrap(action: ContinueAction) -> Result<PromptAction> {
        Ok(PromptAction::ContinueAction(action))
    }

    fn enter(self, _: &mut GameState) -> Result<Option<RaidState>> {
        Ok(None)
    }

    fn actions(self, _: &GameState) -> Result<Vec<ContinueAction>> {
        Ok(vec![ContinueAction::Advance, ContinueAction::Retreat])
    }

    fn active_side(self) -> Side {
        Side::Champion
    }

    fn handle_action(
        self,
        game: &mut GameState,
        action: ContinueAction,
    ) -> Result<Option<RaidState>> {
        match action {
            ContinueAction::Advance => Ok(Some(RaidState::Encounter)),
            ContinueAction::Retreat => {
                mutations::end_raid(game, RaidOutcome::Failure)?;
                Ok(None)
            }
        }
    }

    fn display_state(self, game: &GameState) -> Result<RaidDisplayState> {
        let defenders = game.defender_list(game.raid()?.target);
        Ok(RaidDisplayState::Defenders(defenders[0..=game.raid_encounter()?].to_vec()))
    }

    fn prompt_context(self) -> Option<PromptContext> {
        Some(PromptContext::RaidAdvance)
    }
}
