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

//! Handler for interactive card prompts

use anyhow::Result;
use data::delegates::RaidOutcome;
use data::game::GameState;
use data::game_actions::CardPromptAction;
use data::primitives::Side;

use crate::mana::ManaPurpose;
use crate::{mana, mutations};

pub fn handle(game: &mut GameState, _side: Side, action: CardPromptAction) -> Result<()> {
    match action {
        CardPromptAction::LoseMana(side, amount) => {
            mana::spend(game, side, ManaPurpose::PayForTriggeredAbility, amount)?;
        }
        CardPromptAction::LoseActions(side, amount) => {
            mutations::spend_action_points(game, side, amount)?;
        }
        CardPromptAction::EndRaid => {
            mutations::end_raid(game, RaidOutcome::Failure)?;
        }
        CardPromptAction::TakeDamage(ability_id, amount) => {
            mutations::deal_damage(game, ability_id, amount)?;
        }
        CardPromptAction::TakeDamageEndRaid(ability_id, amount) => {
            mutations::deal_damage(game, ability_id, amount)?;
            mutations::end_raid(game, RaidOutcome::Failure)?;
        }
    }
    Ok(())
}
