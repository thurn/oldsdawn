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
use data::delegates::{
    Delegate, EventData, Flag, GetManaCostQuery, MutationFn, OnDawnEvent, QueryData, RequirementFn,
    Scope,
};
use data::game::GameState;
use data::primitives::{AbilityId, AbilityIndex, BoostCount, CardId, ManaValue};

pub fn invoke2<D: Copy, E: EventData<D>>(game: &mut GameState, event: E) {
    let span = event.span();
    for card_id in game.all_card_ids() {
        let definition = crate::get(game.card(card_id).name);
        for (index, ability) in definition.abilities.iter().enumerate() {
            let scope = Scope::new(AbilityId::new(card_id, index));
            for delegate in &ability.delegates {
                if let Some(functions) = E::get(delegate) {
                    let data = event.data();
                    if (functions.requirement)(game, scope, data) {
                        (functions.mutation)(game, scope, data)
                    }
                }
            }
        }
    }
}

pub fn query2<D: Copy, R, E: QueryData<D, R>>(
    game: &mut GameState,
    event: E,
    initial_value: R,
) -> R {
    let span = event.span();
    let mut result = initial_value;
    for card_id in game.all_card_ids() {
        let definition = crate::get(game.card(card_id).name);
        for (index, ability) in definition.abilities.iter().enumerate() {
            let scope = Scope::new(AbilityId::new(card_id, index));
            for delegate in &ability.delegates {
                if let Some(functions) = E::get(delegate) {
                    let data = event.data();
                    if (functions.requirement)(game, scope, data) {
                        result = (functions.transformation)(game, scope, data, result);
                    }
                }
            }
        }
    }
    result
}

pub fn test(game: &mut GameState, card_id: CardId) {
    invoke2(game, OnDawnEvent(8));
}

pub fn test2(game: &mut GameState, card_id: CardId) -> Option<ManaValue> {
    query2(game, GetManaCostQuery(card_id), Some(3))
}

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

fn delegate_name(game: &GameState) -> &'static str {
    "CanPlayCard"
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
