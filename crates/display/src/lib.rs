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
use data::primitives::{Side, UserId};
use once_cell::sync::Lazy;
use protos::spelldawn::CommandList;

use crate::full_sync::FullSync;

pub mod animations;
pub mod assets;
pub mod diff;
pub mod full_sync;
pub mod rules_text;

static RESPONSES: Lazy<DashMap<UserId, FullSync>> = Lazy::new(DashMap::new);

pub fn connect(game: &GameState, user_side: Side) -> CommandList {
    let user_id = game.player(user_side).id;
    let sync = full_sync::run(game, user_side);
    let mut commands = vec![];
    diff::execute(&mut commands, game, None, &sync);
    RESPONSES.insert(user_id, sync);
    CommandList { commands }
}

pub fn render_updates(game: &GameState, user_side: Side) -> CommandList {
    let updates = game.updates.list().expect("Update tracking is not enabled");
    let user_id = game.player(user_side).id;
    let mut commands = vec![];

    for update in updates.iter().filter(|u| u.is_early_update()) {
        animations::render(&mut commands, *update, game, user_side);
    }

    let sync = full_sync::run(game, user_side);
    let previous_response = RESPONSES
        .get(&user_id)
        .unwrap_or_else(|| panic!("Previous response not found for {:?}", user_id))
        .value();
    diff::execute(&mut commands, game, Some(previous_response), &sync);
    // diff::execute(&mut commands, RESPONSES.get(&user_id).map(|r| r.value()),
    // &sync);
    RESPONSES.insert(user_id, sync);

    for update in updates.iter().filter(|u| !u.is_early_update()) {
        animations::render(&mut commands, *update, game, user_side);
    }

    CommandList { commands }
}
