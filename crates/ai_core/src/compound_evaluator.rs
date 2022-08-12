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

use crate::game_state_node::{GameStateNode, GameStatus};
use crate::state_evaluator::StateEvaluator;

/// A StateEvaluator which combines together the results of a list of child
/// evaluators, multiplying each result by the associated weight.
///
/// Automatically handles the 'game over' state by returning i32::MAX/i32::MIN
/// if the player won/lost the game.
pub struct CompoundEvaluator<TNode: GameStateNode> {
    pub evaluators: Vec<(i32, Box<dyn StateEvaluator<TNode>>)>,
}

impl<TNode: GameStateNode> StateEvaluator<TNode> for CompoundEvaluator<TNode> {
    fn evaluate(&self, node: &TNode, player: TNode::PlayerName) -> Result<i32> {
        if let GameStatus::Completed { winner } = node.status() {
            return if winner == player { Ok(i32::MAX) } else { Ok(i32::MIN) };
        }

        let mut score = 0;
        for (weight, evaluator) in &self.evaluators {
            score += weight * evaluator.evaluate(node, player)?;
        }
        Ok(score)
    }
}
