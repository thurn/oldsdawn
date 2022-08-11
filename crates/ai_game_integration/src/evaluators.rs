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

use ai_core::game_state_node::{GameStateNode, GameStatus};
use ai_core::state_evaluator::StateEvaluator;
use anyhow::Result;
use data::primitives::Side;

use crate::state_node::SpelldawnState;

pub struct ScoreEvaluator {}

impl StateEvaluator<SpelldawnState> for ScoreEvaluator {
    fn evaluate(&self, node: &SpelldawnState, side: Side) -> Result<i32> {
        match node.status() {
            GameStatus::Completed { winner } => {
                if winner == side {
                    Ok(i32::MAX)
                } else {
                    Ok(i32::MIN)
                }
            }
            GameStatus::InProgress { .. } => {
                Ok((node.player(side).score as i32) - (node.player(side.opponent()).score as i32))
            }
        }
    }
}
