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
use with_error::WithError;

use crate::game_state_node::GameStateNode;
use crate::state_evaluator::StateEvaluator;
use crate::state_predictor::StatePredictor;

/// StateCombiners merge together the results of [StatePredictor]s. Given the
/// set of predicted game states, they select the best *actual* game state to
/// use as the basis for a selection algorithm.
pub type StateCombiner<TNode, TEvaluator> =
    fn(&TNode, StatePredictor<TNode>, &TEvaluator) -> Result<TNode>;

/// A pessimistic [StateCombiner]. Evaluates each predicted game state and picks
/// the one which appears to be the worst-case scenario (i.e. has the lowest
/// [StateEvaluator] score) for the current player.
pub fn worst_case<TEvaluator, TNode>(
    node: &TNode,
    predictor: StatePredictor<TNode>,
    evaluator: &TEvaluator,
) -> Result<TNode>
where
    TEvaluator: StateEvaluator<TNode>,
    TNode: GameStateNode,
{
    let player = node.current_turn().with_error(|| "Game is over")?;
    let mut worst = i64::MAX;
    let mut worst_state: Option<TNode> = None;
    for state in predictor(node) {
        let evaluation = evaluator.evaluate(&state, player);
        if evaluation < worst {
            worst = evaluation;
            worst_state = Some(state);
        }
    }

    worst_state.with_error(|| "No state found")
}
