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

//! Core functions of the Delegate system. See the module-level comment in
//! `delegates.rs` for more information about this system.

use std::fmt::Debug;

use data::delegates::{EventData, QueryData, Scope};
use data::game::GameState;
use data::primitives::AbilityId;
use tracing::{info, instrument};

/// Called when a game event occurs, invokes each registered
/// [data::delegates::Delegate] for this event to mutate the [GameState]
/// appropriately.
#[instrument(skip(game))]
pub fn invoke_event<D: Copy + Debug, E: EventData<D>>(game: &mut GameState, event: E) {
    for card_id in game.all_card_ids() {
        let definition = crate::get(game.card(card_id).name);
        for (index, ability) in definition.abilities.iter().enumerate() {
            let scope = Scope::new(AbilityId::new(card_id, index));
            for delegate in &ability.delegates {
                if let Some(functions) = E::get(delegate) {
                    let data = event.data();
                    if (functions.requirement)(game, scope, data) {
                        info!(?event, ?scope, "invoke_event");
                        (functions.mutation)(game, scope, data)
                    }
                }
            }
        }
    }
}

/// Called when game state information is needed, invokes each reigistered
/// [data::delegates::Delegate] for this query and allows them to intercept &
/// transform the final result.
#[instrument(skip(game))]
pub fn perform_query<D: Copy + Debug, R: Debug, E: QueryData<D, R>>(
    game: &GameState,
    query: E,
    initial_value: R,
) -> R {
    let mut result = initial_value;
    for card_id in game.all_card_ids() {
        let definition = crate::get(game.card(card_id).name);
        for (index, ability) in definition.abilities.iter().enumerate() {
            let scope = Scope::new(AbilityId::new(card_id, index));
            for delegate in &ability.delegates {
                if let Some(functions) = E::get(delegate) {
                    let data = query.data();
                    if (functions.requirement)(game, scope, data) {
                        info!(?query, ?scope, "perform_query");
                        result = (functions.transformation)(game, scope, data, result);
                    }
                }
            }
        }
    }
    result
}
