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

use model::card_name::CardName;
use model::delegates::{Context, Delegate};
use model::game::GameState;
use model::primitives::{AbilityId, AbilityIndex, CardId};

pub fn invoke_event<T: Copy>(
    game: &mut GameState,
    event: fn(&mut GameState, Context, Delegate, T),
    data: T,
) {
    for card_id in game.card_ids() {
        let definition = crate::get(game.card(card_id).name);
        for (index, ability) in definition.abilities.iter().enumerate() {
            let context = Context::new(game, AbilityId::new(card_id, index));
            for delegate in &ability.delegates {
                event(game, context, *delegate, data)
            }
        }
    }
}

pub fn perform_query<T: Copy, R>(
    game: &mut GameState,
    query: fn(&GameState, Context, Delegate, T, R) -> R,
    data: T,
    initial_value: R,
) -> R {
    let mut result = initial_value;
    for card_id in game.card_ids() {
        let definition = crate::get(game.card(card_id).name);
        for (index, ability) in definition.abilities.iter().enumerate() {
            let context = Context::new(game, AbilityId::new(card_id, index));
            for delegate in &ability.delegates {
                result = query(game, context, *delegate, data, result)
            }
        }
    }
    result
}
