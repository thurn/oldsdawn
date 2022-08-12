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

use std::cmp;
use std::time::Instant;

use ai_core::agent::AgentConfig;
use ai_core::game_state_node::{GameStateNode, GameStatus};
use ai_core::selection_algorithm::SelectionAlgorithm;
use ai_core::state_evaluator::StateEvaluator;
use anyhow::Result;

use crate::scored_action::ScoredAction;

/// Implements alpha-beta pruning over minimax tree search.
///
/// This is a 'fail soft' implementation per wikipedia. I have not been able to
/// detect any performance or gameplay difference with the 'fail hard' version.
///
/// See <https://en.wikipedia.org/wiki/Alpha-beta_pruning>
pub struct AlphaBetaAlgorithm {
    pub search_depth: u32,
}

impl SelectionAlgorithm for AlphaBetaAlgorithm {
    fn pick_action<N, E>(
        &self,
        config: AgentConfig,
        node: &N,
        evaluator: &E,
        player: N::PlayerName,
    ) -> Result<N::Action>
    where
        N: GameStateNode,
        E: StateEvaluator<N>,
    {
        assert!(matches!(node.status(), GameStatus::InProgress { .. }));
        run_internal(config, node, evaluator, self.search_depth, player, i32::MIN, i32::MAX)?
            .action()
    }
}

fn run_internal<N, E>(
    config: AgentConfig,
    node: &N,
    evaluator: &E,
    depth: u32,
    player: N::PlayerName,
    mut alpha: i32,
    mut beta: i32,
) -> Result<ScoredAction<N::Action>>
where
    N: GameStateNode,
    E: StateEvaluator<N>,
{
    Ok(match node.status() {
        _ if depth == 0 => ScoredAction::new(evaluator.evaluate(node, player)?),
        GameStatus::Completed { .. } => ScoredAction::new(evaluator.evaluate(node, player)?),
        GameStatus::InProgress { current_turn } if current_turn == player => {
            let mut result = ScoredAction::new(i32::MIN);
            for action in node.legal_actions(current_turn)? {
                if deadline_exceeded(config, depth) {
                    return Ok(result.with_fallback_action(action));
                }
                let mut child = node.make_copy();
                child.execute_action(current_turn, action)?;
                let score =
                    run_internal(config, &child, evaluator, depth - 1, player, alpha, beta)?
                        .score();
                alpha = cmp::max(alpha, score);
                result.insert_max(action, score);
                if score >= beta {
                    break; // Beta cutoff
                }
            }
            result
        }
        GameStatus::InProgress { current_turn } => {
            let mut result = ScoredAction::new(i32::MAX);
            for action in node.legal_actions(current_turn)? {
                if deadline_exceeded(config, depth) {
                    return Ok(result.with_fallback_action(action));
                }
                let mut child = node.make_copy();
                child.execute_action(current_turn, action)?;
                let score =
                    run_internal(config, &child, evaluator, depth - 1, player, alpha, beta)?
                        .score();
                beta = cmp::min(beta, score);
                result.insert_min(action, score);
                if score <= alpha {
                    break; // Alpha cutoff
                }
            }
            assert!(result.has_action());
            result
        }
    })
}

/// Check whether `deadline` has been exceeded. Only checks deadlines for higher
/// parts of the tree to avoid excessive calls to Instant::now().
fn deadline_exceeded(config: AgentConfig, depth: u32) -> bool {
    let exceeded = depth > 1 && config.deadline < Instant::now();
    if exceeded && config.panic_on_search_timeout {
        panic!("Search deadline exceeded!");
    }
    exceeded
}
