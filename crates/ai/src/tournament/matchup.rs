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

use data::agent_definition::AgentName;
use data::game::{GamePhase, GameState};
use data::primitives::{PointsValue, Side, TurnNumber};
use enum_iterator::IntoEnumIterator;
use rules::{actions, queries};

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
pub fn run(mut game: GameState, print_actions: bool) -> MatchOutcome {
    loop {
        for side in Side::into_enum_iter() {
            if let GamePhase::GameOver(data) = &game.data.phase {
                return MatchOutcome {
                    winner: OutcomePlayer {
                        agent: game.player(data.winner).agent.unwrap().name,
                        side: data.winner,
                        score: game.player(data.winner).score,
                    },
                    loser: OutcomePlayer {
                        agent: game.player(data.winner.opponent()).agent.unwrap().name,
                        side: data.winner.opponent(),
                        score: game.player(data.winner.opponent()).score,
                    },
                    turn_count: game.data.turn.turn_number,
                };
            }

            if queries::can_take_action(&game, side) {
                let agent_data = game.player(side).agent.expect("Agent");
                let agent = ai::core::get_agent(agent_data.name);
                let state_predictor =
                    ai::core::get_game_state_predictor(agent_data.state_predictor);
                let action = agent(state_predictor(&game, side), side).expect("Agent Error");

                if print_actions {
                    println!("{:?} action: {:?}", side, action);
                }
                actions::handle_user_action(&mut game, side, action).expect("Action Error");
            }
        }
    }
}
