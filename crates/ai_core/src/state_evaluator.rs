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

use crate::game_state_node::GameStateNode;

/// A trait implementation for producing a 'score' for a given game state.
pub trait StateEvaluator<TNode: GameStateNode> {
    /// Evaluate the heuristic value of the given game state for the provided
    /// `player`, returning a higher number for "better" game states.
    ///
    /// Evaluators are expected to return valid results for all game states,
    /// regardless of whether the game has ended, but can return 0 as a default
    /// response.
    ///
    /// For example, the simplest evaluator might return 0 for all in-progress
    /// game states and 1 or -1 based on whether the indicated player won
    /// the game.
    fn evaluate(&self, node: &TNode, player: TNode::PlayerName) -> Result<i32>;
}

/// An evaluator which always returns 0.
pub struct ZeroEvaluator {}

impl<TNode: GameStateNode> StateEvaluator<TNode> for ZeroEvaluator {
    fn evaluate(&self, _: &TNode, _: TNode::PlayerName) -> Result<i32> {
        Ok(0)
    }
}
