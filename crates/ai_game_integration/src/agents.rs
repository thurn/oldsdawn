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

use ai_core::agent::{Agent, AgentData};
use ai_monte_carlo::monte_carlo::{MonteCarloAlgorithm, RandomPlayoutEvaluator};
use ai_monte_carlo::uct1::Uct1;
use ai_tree_search::alpha_beta::AlphaBetaAlgorithm;
use ai_tree_search::minimax::MinimaxAlgorithm;
use anyhow::Result;
use data::game_actions::UserAction;
use data::player_name::NamedPlayer;
use with_error::fail;

use crate::evaluators::ScoreEvaluator;
use crate::state_node::SpelldawnState;

pub fn get(name: NamedPlayer) -> Box<dyn Agent<SpelldawnState>> {
    match name {
        NamedPlayer::TestNoAction => Box::new(NO_ACTION_AGENT),
        NamedPlayer::TestMinimax => Box::new(MINIMAX_AGENT),
        NamedPlayer::TestAlphaBeta => Box::new(ALPHA_BETA_AGENT),
        NamedPlayer::TestUct1 => Box::new(UCT1_AGENT),
    }
}

pub const NO_ACTION_AGENT: NoActionAgent = NoActionAgent {};

pub const MINIMAX_AGENT: AgentData<MinimaxAlgorithm, ScoreEvaluator, SpelldawnState> =
    AgentData::omniscient("MINIMAX", MinimaxAlgorithm { search_depth: 4 }, ScoreEvaluator {});

pub const ALPHA_BETA_AGENT: AgentData<AlphaBetaAlgorithm, ScoreEvaluator, SpelldawnState> =
    AgentData::omniscient("ALPHA_BETA", AlphaBetaAlgorithm { search_depth: 4 }, ScoreEvaluator {});

pub const UCT1_AGENT: AgentData<MonteCarloAlgorithm<Uct1>, RandomPlayoutEvaluator, SpelldawnState> =
    AgentData::omniscient(
        "UCT1",
        MonteCarloAlgorithm { child_score_algorithm: Uct1 {} },
        RandomPlayoutEvaluator {},
    );

pub struct NoActionAgent {}

impl Agent<SpelldawnState> for NoActionAgent {
    fn name(&self) -> &'static str {
        "NO_ACTION"
    }

    fn pick_action(&self, _: Instant, _: &SpelldawnState) -> Result<UserAction> {
        fail!("No Action")
    }
}
