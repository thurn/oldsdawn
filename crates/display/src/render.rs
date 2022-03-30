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

use dashmap::DashMap;
use data::game::GameState;
use data::primitives::{PlayerId, Side};
use data::updates::{GameUpdate, GameUpdateKind};
use once_cell::sync::Lazy;
use protos::spelldawn::game_command::Command;

use crate::full_sync::FullSync;
use crate::response_builder::{CardUpdateTypes, ResponseBuilder};
use crate::{animations, diff, full_sync};

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
    let mut builder =
        ResponseBuilder::new(user_side, CardUpdateTypes::default(), false /* animate */);
    diff::execute(&mut builder, game, None, &sync);
    RESPONSES.insert(user_id, sync);
    builder.build()
}

pub fn render_updates(game: &GameState, user_side: Side) -> Vec<Command> {
    let mut updates = game.updates.list().expect("Update tracking is not enabled").clone();
    updates.sort_by_key(GameUpdate::kind);
    let mut card_update_types = CardUpdateTypes::default();
    for update in &updates {
        animations::populate_card_update_types(game, update, &mut card_update_types);
    }

    let user_id = game.player(user_side).id;
    let mut builder = ResponseBuilder::new(user_side, card_update_types, true /* animate */);

    let sync = full_sync::run(game, user_side);

    for update in &updates {
        if update.kind() == GameUpdateKind::GeneralUpdate {
            let reference = RESPONSES.get(&user_id);
            let previous_response = reference.as_deref();
            diff::execute(&mut builder, game, previous_response, &sync);
        } else {
            animations::render(&mut builder, update, game, user_side);
        }
    }

    RESPONSES.insert(user_id, sync);
    builder.build()
}
