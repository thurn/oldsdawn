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

use ai_core::agent;
use ai_core::agent::{Agent, AgentData};
use ai_monte_carlo::monte_carlo::{MonteCarloAlgorithm, RandomPlayoutEvaluator};
use ai_testing::nim;
use ai_testing::nim::NimState;

#[test]
pub fn uct1() {
    let agent = AgentData::omniscient("UCT1", MonteCarloAlgorithm {}, RandomPlayoutEvaluator {});

    nim::assert_perfect_short(&NimState::new(2), &agent);
    nim::assert_perfect_short(&NimState::new_with_piles(2, 2, 3), &agent);
    nim::assert_perfect_short(&NimState::new(3), &agent);
    nim::assert_perfect_short(&NimState::new_with_piles(1, 1, 3), &agent);
    nim::assert_perfect_short(&NimState::new_with_piles(4, 3, 2), &agent);
}

#[test]
pub fn uct1_deadline_exceeded() {
    let agent = AgentData::omniscient("UCT1", MonteCarloAlgorithm {}, RandomPlayoutEvaluator {});
    let state = NimState::new(100);
    let start_time = Instant::now();
    let action = agent.pick_action(agent::deadline(1), &state);
    assert!(action.is_ok());
    assert!(start_time.elapsed().as_secs() < 2);
}
