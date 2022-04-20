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

//! Core definition of AI Agents

use anyhow::Result;
use data::game::GameState;
use data::game_actions::UserAction;
use data::primitives::Side;
use ordered_float::NotNan;

#[derive(Debug, Clone)]
pub struct StatePrediction {
    pub score: f64,
    pub state: GameState,
}

pub type StatePredictionIterator = Box<dyn Iterator<Item = StatePrediction>>;

// It is an error for a predictor to return no game states
pub type GameStatePredictor = fn(&GameState, Side) -> StatePredictionIterator;

// It is an error for an agent to be invoked if there is no legal game action
// available.
pub type Agent = fn(StatePredictionIterator, Side) -> Result<UserAction>;

/// Wraps a float in a [NotNan]. Panics if the input is `NaN`.
pub fn notnan(value: f64) -> NotNan<f64> {
    NotNan::new(value).unwrap()
}
