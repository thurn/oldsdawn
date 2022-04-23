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

//! Contains definitions for configuration of AI Agents

use serde::{Deserialize, Serialize};

/// Identifies different possible Game State Predictors. See the 'agents' crate
/// for more information.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum GameStatePredictorName {
    Omniscient,
}

/// Identifies different possible Agents. See the 'agents' crate for more
/// information.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum AgentName {
    PickFirstAction,
    AlphaBeta,
    MonteCarlo,
}

/// Primary configuration for an AI Agent. See the 'agents' crate for more
/// information.
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq)]
pub struct AgentData {
    pub name: AgentName,
    pub state_predictor: GameStatePredictorName,
}
