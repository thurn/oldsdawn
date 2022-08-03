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

use ai_core::agent::AgentData;
use ai_tree_search::minimax::MinimaxAlgorithm;
use ai_tree_search::single_level::SingleLevel;

use crate::nim::{NimPerfectEvaluator, NimState, NimWinLossEvaluator};

pub enum NimAgentName {
    Perfect,
    Minimax,
}

/// Agent which always makes optimal moves
pub const NIM_PERFECT_AGENT: AgentData<SingleLevel, NimPerfectEvaluator, NimState> =
    AgentData::omniscient("PERFECT", SingleLevel {}, NimPerfectEvaluator {});

pub const NIM_MINIMAX_AGENT: AgentData<MinimaxAlgorithm, NimWinLossEvaluator, NimState> =
    AgentData::omniscient("MINIMAX", MinimaxAlgorithm { search_depth: 25 }, NimWinLossEvaluator {});
