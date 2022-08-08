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

/// Operation mode for child scoring.
#[derive(Debug, Eq, PartialEq, Clone, Copy)]
pub enum SelectionMode {
    /// Balance between trying new children and re-visiting existing children.
    Exploration,
    /// Select the best overall child without giving any weight to exploration.
    Best,
}

/// Trait for selecting which child node of the Monte Carlo search tree to
/// explore. The child which returns the highest score is selected. Inputs are
/// the number of visits to the current parent, number of visits to this child,
/// known reward value for this child, and [SelectionMode].
pub trait ChildScoreAlgorithm {
    fn score(
        &self,
        parent_visits: f64,
        child_visits: f64,
        child_reward: f64,
        selection_mode: SelectionMode,
    ) -> f64;
}
