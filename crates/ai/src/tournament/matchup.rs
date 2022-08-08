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

use std::fmt::{Display, Formatter};

use actions;
use anyhow::Result;
use data::agent_definition::AgentName;
use data::game::{GamePhase, GameState};
use data::primitives::{PointsValue, Side, TurnNumber};
use with_error::WithError;

use crate::tournament::run_tournament::RunGames;

pub struct OutcomePlayer {
    pub agent: AgentName,
    pub side: Side,
    pub score: PointsValue,
}

pub struct MatchOutcome {
    pub winner: OutcomePlayer,
    pub loser: OutcomePlayer,
    pub turn_count: TurnNumber,
}

impl Display for MatchOutcome {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} as {:?} defeats {:?} {:?}-{:?} in {:?} turns",
            self.winner.agent,
            self.winner.side,
            self.loser.agent,
            self.winner.score,
            self.loser.score,
            self.turn_count
        )
    }
}

/// Runs an AI matchup for a given `game`.
///
/// The game must be configured to use Agents for both players.
pub fn run(mut game: GameState, config: RunGames) -> Result<MatchOutcome> {
    loop {
        for side in enum_iterator::all::<Side>() {
            if let GamePhase::GameOver { winner } = &game.data.phase {
                return Ok(MatchOutcome {
                    winner: OutcomePlayer {
                        agent: game.player(*winner).agent.unwrap().name,
                        side: *winner,
                        score: game.player(*winner).score,
                    },
                    loser: OutcomePlayer {
                        agent: game.player(winner.opponent()).agent.unwrap().name,
                        side: winner.opponent(),
                        score: game.player(winner.opponent()).score,
                    },
                    turn_count: game.data.turn.turn_number,
                });
            }

            if actions::can_take_action(&game, side) {
                let agent_data = game.player(side).agent.with_error(|| "Agent not found")?;
                let agent = crate::core::get_agent(agent_data.name);
                let state_predictor =
                    crate::core::get_game_state_predictor(agent_data.state_predictor);
                let action = agent(state_predictor(&game, side), side)?;

                if config == RunGames::PrintActions {
                    println!("{:?} action: {:?}", side, action);
                }
                actions::handle_user_action(&mut game, side, action)?;
            }
        }
    }
}
