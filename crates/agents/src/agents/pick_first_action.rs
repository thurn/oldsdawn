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

//! The 'first action' agent always selects the first legal game action based on
//! the first predicted game state and the action order returned by the
//! `legal_actions` module.

use anyhow::Result;
use data::agent_definition::AgentName;
use data::game_actions::UserAction;
use data::primitives::Side;
use data::with_error::WithError;
use linkme::distributed_slice;

use crate::core::types::StatePredictionIterator;
use crate::core::{legal_actions, AgentPair, AGENTS};

#[distributed_slice(AGENTS)]
static AGENT: AgentPair = (AgentName::PickFirstAction, pick_first_action);

fn pick_first_action(mut states: StatePredictionIterator, side: Side) -> Result<UserAction> {
    let game = states.next().with_error(|| "Expected at least one GameState")?.state;
    let mut legal_actions = legal_actions::evaluate(&game, side);
    legal_actions.next().with_error(|| "Expected at least one action")
}
