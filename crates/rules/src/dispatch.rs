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

use data::card_name::CardName;
use data::delegates::{Delegate, Scope};
use data::game::GameState;
use data::primitives::{AbilityId, AbilityIndex, CardId};

pub fn invoke_event<T: Copy>(
    game: &mut GameState,
    event: fn(&mut GameState, Scope, &Delegate, T),
    data: T,
) {
    for card_id in game.all_card_ids() {
        let definition = crate::get(game.card(card_id).name);
        for (index, ability) in definition.abilities.iter().enumerate() {
            let scope = Scope::new(AbilityId::new(card_id, index));
            for delegate in &ability.delegates {
                event(game, scope, delegate, data)
            }
        }
    }
}

pub fn perform_query<T: Copy, R>(
    game: &GameState,
    query: fn(&GameState, Scope, &Delegate, T, R) -> R,
    data: T,
    initial_value: R,
) -> R {
    let mut result = initial_value;
    for card_id in game.all_card_ids() {
        let definition = crate::get(game.card(card_id).name);
        for (index, ability) in definition.abilities.iter().enumerate() {
            let scope = Scope::new(AbilityId::new(card_id, index));
            for delegate in &ability.delegates {
                result = query(game, scope, delegate, data, result)
            }
        }
    }
    result
}
