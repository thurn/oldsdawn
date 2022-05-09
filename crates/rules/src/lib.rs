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

//! All primary game rules, responses to user actions, and associated helpers

use std::collections::HashMap;

use data::card_definition::{Ability, CardDefinition};
use data::card_name::CardName;
use data::game::GameState;
use data::primitives::{AbilityId, CardId};
use linkme::distributed_slice;
use once_cell::sync::Lazy;

pub mod abilities;
pub mod actions;
pub mod card_text;
pub mod dispatch;
pub mod flags;
pub mod helpers;
pub mod mana;
pub mod mutations;
pub mod queries;
pub mod raid_actions;
pub mod raid_phases;

#[distributed_slice]
pub static DEFINITIONS: [fn() -> CardDefinition] = [..];

/// Contains [CardDefinition]s for all known cards, keyed by [CardName]
pub static CARDS: Lazy<HashMap<CardName, CardDefinition>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for card_fn in DEFINITIONS {
        let card = card_fn();
        map.insert(card.name, card);
    }
    map
});

/// Looks up the definition for a [CardName]. Panics if no such card is defined.
pub fn get(name: CardName) -> &'static CardDefinition {
    CARDS.get(&name).unwrap_or_else(|| panic!("Card not found: {:?}", name))
}

pub fn card_definition(game: &GameState, card_id: CardId) -> &'static CardDefinition {
    get(game.card(card_id).name)
}

pub fn ability_definition(game: &GameState, ability_id: AbilityId) -> &'static Ability {
    card_definition(game, ability_id.card_id).ability(ability_id.index)
}
