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
use data::fail;
use data::game::{GameState, RaidState};
use data::game_actions::{PromptAction, PromptContext, RoomActivationAction};
use data::primitives::Side;

use crate::raid::core::RaidStateNode;
use crate::raid::defenders;

#[derive(Debug, Clone, Copy)]
pub struct ActivateState {}

impl RaidStateNode<RoomActivationAction> for ActivateState {
    fn unwrap(action: PromptAction) -> Result<RoomActivationAction> {
        match action {
            PromptAction::ActivateRoomAction(action) => Ok(action),
            _ => fail!("Expected RoomActivationAction"),
        }
    }

    fn wrap(action: RoomActivationAction) -> Result<PromptAction> {
        Ok(PromptAction::ActivateRoomAction(action))
    }

    fn enter(self, _: &mut GameState) -> Result<Option<RaidState>> {
        Ok(None)
    }

    fn actions(self, _: &GameState) -> Result<Vec<RoomActivationAction>> {
        Ok(vec![RoomActivationAction::Activate, RoomActivationAction::Pass])
    }

    fn active_side(self) -> Side {
        Side::Overlord
    }

    fn handle_action(
        self,
        game: &mut GameState,
        action: RoomActivationAction,
    ) -> Result<Option<RaidState>> {
        game.raid_mut()?.room_active = action == RoomActivationAction::Activate;
        Ok(Some(if let Some(encounter) = defenders::next_encounter(game, None)? {
            game.raid_mut()?.encounter = Some(encounter);
            RaidState::Encounter
        } else {
            RaidState::Access
        }))
    }

    fn prompt_context(self) -> Option<PromptContext> {
        Some(PromptContext::ActivateRoom)
    }
}
