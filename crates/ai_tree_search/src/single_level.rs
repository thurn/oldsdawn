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

use std::time::Instant;

use ai_core::game_state_node::GameStateNode;
use ai_core::selection_algorithm::SelectionAlgorithm;
use ai_core::state_evaluator::StateEvaluator;
use anyhow::Result;
use with_error::WithError;

/// An agent which does a depth 1 search of legal actions, returning the one
/// that produces the best evaluator state.
pub struct SingleLevel {}

impl SelectionAlgorithm for SingleLevel {
    fn pick_action<N, E>(
        &self,
        _deadline: Instant,
        node: &N,
        evaluator: &E,
        player: N::PlayerName,
    ) -> Result<N::Action>
    where
        N: GameStateNode,
        E: StateEvaluator<N>,
    {
        let mut best_score = i64::MIN;
        let mut best_action: Option<N::Action> = None;

        for action in node.legal_actions()? {
            let mut child = node.make_copy();
            child.execute_action(player, action)?;
            let score = evaluator.evaluate(&child, player);
            if score > best_score {
                best_score = score;
                best_action = Some(action);
            }
        }

        best_action.with_error(|| "No legal actions found")
    }
}
