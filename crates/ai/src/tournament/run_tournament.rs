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

use anyhow::Result;
use cards::{decklists, initialize};
use data::agent_definition::{AgentData, AgentName, GameStatePredictorName};
use data::game::GameState;

use crate::tournament::matchup;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running AI match");
    initialize::run();
    let mut game = decklists::canonical_game()?;
    run_games(&mut game, 10, AgentName::AlphaBeta, AgentName::MonteCarlo, RunGames::PrintActions)?;
    Ok(())
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum RunGames {
    NoPrint,
    PrintActions,
}

pub fn run_games(
    game: &mut GameState,
    count: u32,
    one: AgentName,
    two: AgentName,
    config: RunGames,
) -> Result<()> {
    for _ in 0..count {
        game.overlord.agent =
            Some(AgentData { name: one, state_predictor: GameStatePredictorName::Omniscient });
        game.champion.agent =
            Some(AgentData { name: two, state_predictor: GameStatePredictorName::Omniscient });

        let outcome = matchup::run(game.clone(), config)?;
        if config == RunGames::PrintActions {
            println!(">>> {}", outcome);
        }

        game.overlord.agent.unwrap().name = two;
        game.champion.agent.unwrap().name = one;

        let outcome = matchup::run(game.clone(), config)?;
        if config == RunGames::PrintActions {
            println!(">>> {}", outcome);
        }
    }

    Ok(())
}
