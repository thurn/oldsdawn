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

use ai_core::agent::{Agent, AgentConfig, AgentData};
use ai_core::compound_evaluator::CompoundEvaluator;
use ai_monte_carlo::monte_carlo::{MonteCarloAlgorithm, RandomPlayoutEvaluator};
use ai_monte_carlo::uct1::Uct1;
use ai_tree_search::alpha_beta::AlphaBetaAlgorithm;
use ai_tree_search::minimax::MinimaxAlgorithm;
use anyhow::Result;
use data::game_actions::UserAction;
use data::player_name::NamedPlayer;
use with_error::fail;

use crate::evaluators::{
    CardsInHandEvaluator, CardsInPlayEvaluator, LevelCountersEvaluator, ManaDifferenceEvaluator,
    ScoreEvaluator,
};
use crate::state_node::SpelldawnState;

pub fn get(name: NamedPlayer) -> Box<dyn Agent<SpelldawnState>> {
    match name {
        NamedPlayer::TestNoAction => Box::new(NoActionAgent {}),
        NamedPlayer::TestMinimax => Box::new(AgentData::omniscient(
            "MINIMAX",
            MinimaxAlgorithm { search_depth: 4 },
            ScoreEvaluator {},
        )),
        NamedPlayer::TestAlphaBetaScores => Box::new(AgentData::omniscient(
            "ALPHA_BETA_SCORES",
            AlphaBetaAlgorithm { search_depth: 4 },
            CompoundEvaluator { evaluators: vec![(1, Box::new(ScoreEvaluator {}))] },
        )),
        NamedPlayer::TestAlphaBetaHeuristics => Box::new(AgentData::omniscient(
            "ALPHA_BETA_HEURISTICS",
            AlphaBetaAlgorithm { search_depth: 4 },
            CompoundEvaluator {
                evaluators: vec![
                    (100_000, Box::new(ScoreEvaluator {})),
                    (10, Box::new(ManaDifferenceEvaluator {})),
                    (5, Box::new(CardsInHandEvaluator {})),
                    (15, Box::new(CardsInPlayEvaluator {})),
                    (20, Box::new(LevelCountersEvaluator {})),
                ],
            },
        )),
        NamedPlayer::TestUct1 => Box::new(AgentData::omniscient(
            "UCT1",
            MonteCarloAlgorithm { child_score_algorithm: Uct1 {} },
            RandomPlayoutEvaluator {},
        )),
    }
}

pub struct NoActionAgent {}

impl Agent<SpelldawnState> for NoActionAgent {
    fn name(&self) -> &'static str {
        "NO_ACTION"
    }

    fn pick_action(&self, _: AgentConfig, _: &SpelldawnState) -> Result<UserAction> {
        fail!("No Action")
    }
}
