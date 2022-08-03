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

use std::iter;

use crate::game_state_node::GameStateNode;

/// StatePredictors address the problem of hidden information in games. Most
/// decision rules function effectively only for perfect-information games. The
/// agent system allows a StatePredictor to be defined which enumerates many
/// *possible* states which a game could currently be in given its actual
/// canonical game state.
///
/// The simplest StatePredictor is the [omniscient] predictor, which simply
/// returns the actual canonical game state with all hidden information
/// revealed. This is obviously the most effective approach in terms of AI
/// performance, but it is effectively cheating.
pub type StatePredictor<TNode> = fn(&TNode) -> Box<dyn Iterator<Item = TNode>>;

/// A [StatePredictor] which returns the actual canonical game state as the only
/// state prediction.
///
/// In other words, this creates an agent with perfect information about hidden
/// game state, i.e. one who cheats.
pub fn omniscient<N: 'static>(node: &N) -> Box<dyn Iterator<Item = N>>
where
    N: GameStateNode,
{
    Box::new(iter::once(node.make_copy()))
}
