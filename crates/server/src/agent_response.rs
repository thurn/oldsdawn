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

//! Functions  for providing AI responses to the user

use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use anyhow::Result;
use concurrent_queue::ConcurrentQueue;
use data::agent_definition::AgentData;
use data::fail;
use data::game::GameState;
use data::primitives::{GameId, PlayerId, Side};
use display::adapters;
use enum_iterator::IntoEnumIterator;
use once_cell::sync::Lazy;
use protos::spelldawn::{CommandList, GameRequest};
use rules::queries;
use tracing::warn;

use crate::database::{Database, SledDatabase};
use crate::requests;

/// Queue of agent responses that need to be sent to the client
pub static RESPONSES: Lazy<ConcurrentQueue<CommandList>> = Lazy::new(ConcurrentQueue::unbounded);

// This feels safe-ish, should be able to ignore everything after an agent is
// active?
static AGENT_RUNNING: AtomicBool = AtomicBool::new(false);

pub fn handle_request(database: SledDatabase, request: &GameRequest) -> Result<()> {
    let game_id = match adapters::to_optional_server_game_id(&request.game_id) {
        None => return Ok(()),
        Some(game_id) => game_id,
    };
    let respond_to = adapters::to_server_player_id(request.player_id)?;

    let game = database.game(game_id)?;

    if active_agent(&game).is_some() && !AGENT_RUNNING.fetch_nand(false, Ordering::Relaxed) {
        thread::spawn(move || {
            run_agent_loop(database, game_id, respond_to).expect("Error running agent");
            AGENT_RUNNING.store(false, Ordering::Relaxed);
        });
    }

    Ok(())
}

/// Returns a ([Side], [AgentData]) tuple for an agent that can currently act in
/// this game, if one exists.
fn active_agent(game: &GameState) -> Option<(Side, AgentData)> {
    for side in Side::into_enum_iter() {
        if let Some(data) = game.player(side).agent {
            if queries::can_take_action(game, side) {
                return Some((side, data));
            }
        }
    }
    None
}

/// Checks if an AI response is required for the game described by `request`, if
/// any, and applies AI actions if needed.
pub fn check_for_agent_response(database: SledDatabase, request: &GameRequest) -> Result<()> {
    if let Some(game_id) = adapters::to_optional_server_game_id(&request.game_id) {
        if database.has_game(game_id)? {
            tokio::spawn(async move {
                run_deprecated_agent_loop(database, game_id).await.expect("Agent error");
            });
        }
    }

    Ok(())
}

fn run_agent_loop(mut database: SledDatabase, game_id: GameId, respond_to: PlayerId) -> Result<()> {
    loop {
        let game = database.game(game_id)?;
        if let Some((side, agent_data)) = active_agent(&game) {
            let agent = ai::core::get_agent(agent_data.name);
            let state_predictor = ai::core::get_game_state_predictor(agent_data.state_predictor);
            let action = agent(state_predictor(&game, side), side)?;
            let response = requests::handle_action(
                &mut database,
                game.player(side).id,
                Some(game_id),
                action,
            )?;

            match response.opponent_response {
                Some((oid, response)) if oid == respond_to => {
                    RESPONSES.push(response)?;
                }
                _ if game.player(side).id == respond_to => {
                    RESPONSES.push(response.command_list)?;
                }
                _ => {
                    fail!("Unknown PlayerId {:?}", respond_to);
                }
            }
        } else {
            return Ok(());
        }
    }
}

async fn run_deprecated_agent_loop(mut database: SledDatabase, game_id: GameId) -> Result<()> {
    loop {
        let mut took_action = false;
        for side in Side::into_enum_iter() {
            let game = database.game(game_id)?;
            if let Some(agent_data) = game.player(side).agent {
                if queries::can_take_action(&game, side) {
                    took_action = true;
                    let agent_name = agent_data.name;
                    let agent = ai::core::get_agent(agent_data.name);
                    let state_predictor =
                        ai::core::get_game_state_predictor(agent_data.state_predictor);
                    warn!(?side, ?agent_name, "running agent");
                    let action = agent(state_predictor(&game, side), side)?;
                    warn!(?side, ?action, "applying agent action");
                    let response = requests::handle_action(
                        &mut database,
                        game.player(side).id,
                        Some(game_id),
                        action,
                    )?;
                    requests::send_player_response(response.opponent_response).await;
                    requests::send_player_response(Some((
                        game.player(side).id,
                        response.command_list,
                    )))
                    .await
                }
            }
        }

        if !took_action {
            return Ok(());
        }
    }
}
