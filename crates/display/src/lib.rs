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

//! Converts internal game state representation into a format the client can
//! understand

use dashmap::DashMap;
use data::game::GameState;
use data::primitives::{PlayerId, Side};
use once_cell::sync::Lazy;
use protos::spelldawn::game_command::Command;

use crate::full_sync::FullSync;
use crate::response_builder::ResponseBuilder;

pub mod adapters;
pub mod animations;
pub mod assets;
pub mod diff;
pub mod full_sync;
pub mod interface;
pub mod prompts;
pub mod response_builder;
pub mod rules_text;

/// Map from user IDs to the most recent game response we sent to that user.
static RESPONSES: Lazy<DashMap<PlayerId, FullSync>> = Lazy::new(DashMap::new);

/// Clears cached responses for the provided [PlayerId].
pub fn on_disconnect(player_id: PlayerId) {
    RESPONSES.remove(&player_id);
}

/// Builds a command list for game connection requests. Executes a full sync of
/// the state of the provided `game`, returning a command to update the state of
/// every active game object. Caches the response for use by future incremental
/// updates via [render_updates].
pub fn connect(game: &GameState, user_side: Side) -> Vec<Command> {
    let user_id = game.player(user_side).id;
    let sync = full_sync::run(game, user_side);
    let mut builder = ResponseBuilder::new(user_side, false /* animate */);
    diff::execute(&mut builder, game, None, &sync);
    RESPONSES.insert(user_id, sync);
    builder.build()
}

/// The central interface-rendering function. Builds a command list for
/// incremental game updates.
///
/// This process operates in three phases:
///
/// 1) A complete view of the state of every object in the game is built via
/// [full_sync::run]
///
/// 2) The complete sync is diffed against the last cached response sent to this
/// user, if any, via [diff::execute]. Only fields which have changed since the
/// last response was sent are included in the client payload.
///
/// 3) Any animations required on top of the basic game object state sync are
/// added, by calling [animations::render]. The animation system is responsible
/// for tasks like moving cards around and playing visual effects.
pub fn render_updates(game: &GameState, user_side: Side) -> Vec<Command> {
    let updates = game.updates.list().expect("Update tracking is not enabled");
    let user_id = game.player(user_side).id;
    let mut builder = ResponseBuilder::new(user_side, true /* animate */);

    let sync = full_sync::run(game, user_side);

    let previous_response = RESPONSES.get(&user_id).map(|r| r.value());
    diff::execute(&mut builder, game, previous_response, &sync);

    RESPONSES.insert(user_id, sync);

    for update in updates {
        animations::render(&mut builder, *update, game, user_side);
    }

    builder.build()
}
