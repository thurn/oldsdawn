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

use crate::agent::AgentConfig;
use crate::game_state_node::GameStateNode;
use crate::state_evaluator::StateEvaluator;

/// A trait for generic decision rules that select a game action to take without
/// specific game knowledge.
pub trait SelectionAlgorithm {
    /// Should return the best action action for the current player `player`
    /// to take in the provided `node` game state, using the provided
    /// `evaluator` to evaluate different game outcomes.
    ///
    /// Implementations are expected to return a result before the
    /// `config.deadline` time by periodically comparing it to
    /// `Instant::now()`.
    fn pick_action<TStateNode, TEvaluator>(
        &self,
        config: AgentConfig,
        node: &TStateNode,
        evaluator: &TEvaluator,
        player: TStateNode::PlayerName,
    ) -> Result<TStateNode::Action>
    where
        TStateNode: GameStateNode,
        TEvaluator: StateEvaluator<TStateNode>;
}
