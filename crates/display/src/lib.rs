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

use std::collections::HashMap;

use dashmap::DashMap;
use data::game::GameState;
use data::primitives::{PlayerId, Side};
use data::updates::GameUpdate;
use once_cell::sync::Lazy;
use protos::spelldawn::CommandList;

use crate::full_sync::FullSync;

pub mod adapters;
pub mod animations;
pub mod assets;
pub mod diff;
pub mod full_sync;
pub mod rules_text;

/// Map from user IDs to the most recent game response we sent to that user.
static RESPONSES: Lazy<DashMap<PlayerId, FullSync>> = Lazy::new(DashMap::new);

/// Builds a command list for game connection requests. Executes a full sync of
/// the state of the provided `game`, returning a command to update the state of
/// every active game object. Caches the response for use by future incremental
/// updates via [render_updates].
pub fn connect(game: &GameState, user_side: Side) -> CommandList {
    let user_id = game.player(user_side).id;
    let sync = full_sync::run(game, user_side, HashMap::new());
    let mut commands = vec![];
    diff::execute(&mut commands, game, None, &sync);
    RESPONSES.insert(user_id, sync);
    CommandList { commands }
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
pub fn render_updates(game: &GameState, user_side: Side) -> CommandList {
    let updates = game.updates.list().expect("Update tracking is not enabled");
    let user_id = game.player(user_side).id;
    let mut commands = vec![];

    // Some animations need to modify the game sync behavior -- for example, for
    // the 'draw card' animation, we cause the card to initially appear on top
    // of the user's deck, even though this card is really in the user's hand.
    let mut card_creation = HashMap::new();
    for update in updates {
        if let GameUpdate::DrawCard(card_id) = update {
            card_creation
                .insert(*card_id, animations::card_draw_creation_strategy(user_side, *card_id));
        }
    }

    let sync = full_sync::run(game, user_side, card_creation);

    let previous_response = RESPONSES
        .get(&user_id)
        .unwrap_or_else(|| panic!("Previous response not found for {:?}", user_id))
        .value();
    diff::execute(&mut commands, game, Some(previous_response), &sync);
    // diff::execute(&mut commands, RESPONSES.get(&user_id).map(|r| r.value()),
    // &sync);

    RESPONSES.insert(user_id, sync);

    for update in updates {
        animations::render(&mut commands, *update, game, user_side);
    }

    CommandList { commands }
}
