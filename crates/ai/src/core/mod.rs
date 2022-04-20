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

//! Primary datatypes and helper functions for implementing AI agents

use data::agent_definition::{AgentName, GameStatePredictorName};
use linkme::distributed_slice;
use types::{Agent, GameStatePredictor};

use crate::agents::{alpha_beta, pick_first_action};
use crate::predictors::omniscient;

pub mod legal_actions;
pub mod types;

pub fn initialize() {
    pick_first_action::initialize();
    alpha_beta::initialize();
    omniscient::initialize();
}

pub type AgentPair = (AgentName, Agent);

#[distributed_slice]
pub static AGENTS: [AgentPair] = [..];

/// Looks up the definition for an [AgentName]. Panics if no such agent is
/// defined.
pub fn get_agent(name: AgentName) -> Agent {
    AGENTS
        .iter()
        .find(|(n, _)| name == *n)
        .unwrap_or_else(|| panic!("Agent not found: {:?}", name))
        .1
}

pub type GameStatePredictorPair = (GameStatePredictorName, GameStatePredictor);

#[distributed_slice]
pub static GAME_STATE_PREDICTORS: [GameStatePredictorPair] = [..];

/// Looks up the definition for a [GameStatePredictorName]. Panics if no such
/// predictor is defined.
pub fn get_game_state_predictor(name: GameStatePredictorName) -> GameStatePredictor {
    GAME_STATE_PREDICTORS
        .iter()
        .find(|(n, _)| name == *n)
        .unwrap_or_else(|| panic!("Predictor not found: {:?}", name))
        .1
}
