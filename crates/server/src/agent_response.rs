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

use data::primitives::Side;
use display::adapters;
use enum_iterator::IntoEnumIterator;
use protos::spelldawn::GameRequest;
use rules::queries;

use crate::database::{Database, SledDatabase};
use crate::requests;

/// Checks if an AI response is required for the game described by `request`, if
/// any, and applies AI actions if needed.
pub fn check_for_agent_response(
    mut database: SledDatabase,
    request: &GameRequest,
) -> anyhow::Result<()> {
    if let Some(game_id) = adapters::to_optional_server_game_id(&request.game_id) {
        if database.has_game(game_id)? {
            tokio::spawn(async move {
                loop {
                    let mut took_action = false;
                    let game = database.game(game_id).expect("game");
                    for side in Side::into_enum_iter() {
                        if let Some(agent_data) = game.player(side).agent {
                            if queries::can_take_action(&game, side) {
                                took_action = true;
                                let agent = ai::core::get_agent(agent_data.name);
                                let state_predictor =
                                    ai::core::get_game_state_predictor(agent_data.state_predictor);
                                let action = agent(state_predictor(&game, side), side)
                                    .expect("Error invoking AI Agent");
                                let response = requests::handle_action(
                                    &mut database,
                                    game.player(side).id,
                                    Some(game_id),
                                    action,
                                )
                                .expect("Error handling GameAction");
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
                        break;
                    }
                }
            });
        }
    }

    Ok(())
}
