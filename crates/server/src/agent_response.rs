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

use ai_core::agent;
use ai_core::agent::Agent;
use ai_game_integration::agents;
use ai_game_integration::state_node::SpelldawnState;
use anyhow::Result;
use concurrent_queue::ConcurrentQueue;
use data::game::GameState;
use data::player_data;
use data::player_name::{NamedPlayer, PlayerId};
use data::primitives::{GameId, Side};
use once_cell::sync::Lazy;
use protos::spelldawn::{CommandList, GameRequest};
use with_error::fail;

use crate::database::Database;
use crate::requests;

// This feels safe-ish?
static AGENT_RUNNING: AtomicBool = AtomicBool::new(false);

/// Queue of agent responses that need to be sent to the client, used in offline
/// mode
pub static RESPONSES: Lazy<ConcurrentQueue<CommandList>> = Lazy::new(ConcurrentQueue::unbounded);

/// What to do with responses produced by the agent.
pub enum HandleRequest {
    /// Send each response to the the player who initiated the `GameRequest`.
    SendToPlayer,

    /// Store each response in the [RESPONSES] queue for use by the plugin.
    PushQueue,
}

pub fn handle_request(
    mut database: impl Database + 'static,
    request: &GameRequest,
    handle_request: HandleRequest,
) -> Result<()> {
    let respond_to = requests::player_id(&mut database, &request.player_id)?;
    let game_id = match player_data::current_game_id(database.player(respond_to)?) {
        Some(game_id) => game_id,
        _ => return Ok(()),
    };
    let game = database.game(game_id)?;

    if active_agent(&game).is_some() && !AGENT_RUNNING.swap(true, Ordering::Relaxed) {
        tokio::spawn(async move {
            run_agent_loop(database, game_id, respond_to, handle_request)
                .await
                .expect("Error running agent");
            AGENT_RUNNING.store(false, Ordering::Relaxed);
        });
    }
    Ok(())
}

/// Returns a ([Side], [AgentData]) tuple for an agent that can currently act in
/// this game, if one exists.
fn active_agent(game: &GameState) -> Option<(Side, Box<dyn Agent<SpelldawnState>>)> {
    for side in enum_iterator::all::<Side>() {
        if let PlayerId::Named(name) = game.player(side).id {
            if name != NamedPlayer::TestNoAction && actions::can_take_action(game, side) {
                return Some((side, agents::get(name)));
            }
        }
    }
    None
}

async fn run_agent_loop(
    mut database: impl Database,
    game_id: GameId,
    respond_to: PlayerId,
    handle_request: HandleRequest,
) -> Result<()> {
    loop {
        let game = SpelldawnState(database.game(game_id)?);
        let commands = if let Some((side, agent)) = active_agent(&game) {
            let action = agent.pick_action(agent::deadline(3), &game)?;
            let response = requests::handle_action(
                &mut database,
                game.player(side).id,
                Some(game_id),
                action,
            )?;

            match response.opponent_response {
                Some((oid, response)) if oid == respond_to => response,
                _ if game.player(side).id == respond_to => response.command_list,
                _ => {
                    fail!("Unknown PlayerId {:?}", respond_to);
                }
            }
        } else {
            return Ok(());
        };

        match handle_request {
            HandleRequest::SendToPlayer => {
                requests::send_player_response(Some((respond_to, commands))).await;
            }
            HandleRequest::PushQueue => {
                RESPONSES.push(commands)?;
            }
        }
    }
}
