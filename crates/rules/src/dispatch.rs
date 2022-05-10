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

use std::collections::HashMap;
use std::fmt::Debug;

use data::delegates::{DelegateCache, DelegateContext, EventData, QueryData, Scope};
use data::game::GameState;
use data::primitives::AbilityId;
use tracing::instrument;

/// Adds a [DelegateCache] for this game in order to improve lookup performance.
pub fn populate_delegate_cache(game: &mut GameState) {
    let mut result = HashMap::new();
    for card_id in game.all_card_ids() {
        let definition = crate::get(game.card(card_id).name);
        for (index, ability) in definition.abilities.iter().enumerate() {
            let ability_id = AbilityId::new(card_id, index);
            let scope = Scope::new(ability_id);
            for delegate in &ability.delegates {
                result
                    .entry(delegate.kind())
                    .or_insert_with(Vec::new)
                    .push(DelegateContext { delegate: delegate.clone(), scope });
            }
        }
    }

    game.delegate_cache = DelegateCache { lookup: result };
}

/// Called when a game event occurs, invokes each registered
/// [data::delegates::Delegate] for this event to mutate the [GameState]
/// appropriately.
#[instrument(skip(game))]
pub fn invoke_event<D: Copy + Debug, E: EventData<D>>(game: &mut GameState, event: E) {
    let count = game.delegate_cache.delegate_count(event.kind());
    for i in 0..count {
        let delegate_context = game.delegate_cache.get(event.kind(), i);
        let scope = delegate_context.scope;
        let functions = E::extract(&delegate_context.delegate).expect("delegate");
        let data = event.data();
        if (functions.requirement)(game, scope, data) {
            (functions.mutation)(game, scope, data);
        }
    }
}

/// Called when game state information is needed. Invokes each registered
/// [data::delegates::Delegate] for this query and allows them to intercept &
/// transform the final result.
#[instrument(skip(game))]
pub fn perform_query<D: Copy + Debug, R: Debug, E: QueryData<D, R>>(
    game: &GameState,
    query: E,
    initial_value: R,
) -> R {
    let mut result = initial_value;
    let count = game.delegate_cache.delegate_count(query.kind());
    for i in 0..count {
        let delegate_context = game.delegate_cache.get(query.kind(), i);
        let scope = delegate_context.scope;
        let functions = E::extract(&delegate_context.delegate).expect("delegate");
        let data = query.data();
        if (functions.requirement)(game, scope, data) {
            result = (functions.transformation)(game, scope, data, result);
        }
    }
    result
}
