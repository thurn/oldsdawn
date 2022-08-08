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

use std::f64::consts;

use crate::child_score::{ChildScoreAlgorithm, SelectionMode};

/// This implements the UCT1 algorithm for child scoring, a standard approach
/// for selecting children and solution to the 'multi-armed bandit' problem.
///
/// Pseudocode:
/// ```text
/// 𝐟𝐮𝐧𝐜𝐭𝐢𝐨𝐧 BESTCHILD(v,c)
///   𝐫𝐞𝐭𝐮𝐫𝐧 argmax(
///     v′ ∈ children of v:
///     Q(v′) / N(v′) +
///     c * √ [ 2 * ln(N(v)) / N(v′) ]
///   )
/// ```
pub struct Uct1 {}

impl ChildScoreAlgorithm for Uct1 {
    fn score(
        &self,
        parent_visits: f64,
        child_visits: f64,
        child_reward: f64,
        selection_mode: SelectionMode,
    ) -> f64 {
        let exploitation = child_reward / child_visits;
        let exploration = f64::sqrt((2.0 * f64::ln(parent_visits)) / child_visits);
        let exploration_bias = match selection_mode {
            SelectionMode::Exploration => consts::FRAC_1_SQRT_2,
            SelectionMode::Best => 0.0,
        };
        exploitation + (exploration_bias * exploration)
    }
}
