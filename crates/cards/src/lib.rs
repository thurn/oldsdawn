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

#![deny(warnings)]
#![deny(clippy::all)]
#![deny(clippy::cast_lossless)]
#![deny(clippy::cloned_instead_of_copied)]
#![deny(clippy::copy_iterator)]
#![deny(clippy::default_trait_access)]
#![deny(clippy::if_then_some_else_none)]
#![deny(clippy::inconsistent_struct_constructor)]
#![deny(clippy::inefficient_to_string)]
#![deny(clippy::integer_division)]
#![deny(clippy::let_underscore_drop)]
#![deny(clippy::let_underscore_must_use)]
#![deny(clippy::manual_ok_or)]
#![deny(clippy::map_flatten)]
#![deny(clippy::map_unwrap_or)]
#![deny(clippy::match_same_arms)]
#![deny(clippy::multiple_inherent_impl)]
#![deny(clippy::needless_continue)]
#![deny(clippy::needless_for_each)]
#![deny(clippy::option_if_let_else)]
#![deny(clippy::redundant_closure_for_method_calls)]
#![deny(clippy::ref_option_ref)]
#![deny(clippy::string_to_string)]
#![deny(clippy::trait_duplication_in_bounds)]
#![deny(clippy::unnecessary_self_imports)]
#![deny(clippy::unnested_or_patterns)]
#![deny(clippy::unused_self)]
#![deny(clippy::unwrap_in_result)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::use_self)]
#![deny(clippy::used_underscore_binding)]
#![deny(clippy::useless_let_if_seq)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_imports)]

use model::card_definition::CardDefinition;
use model::card_name::CardName;
use once_cell::sync::Lazy;
use std::borrow::Borrow;
use std::collections::HashMap;
use std::sync::Mutex;

pub mod card_helpers;
pub mod dispatch;
pub mod queries;

mod abilities;
mod champion_spells;
mod minions;
mod projects;
mod schemes;
mod weapons;

// TODO: Switch back to the linkme crate once https://github.com/dtolnay/linkme/issues/41 is fixed
static DEFINITIONS: &[fn() -> CardDefinition] = &[
    champion_spells::arcane_recovery,
    weapons::greataxe,
    projects::gold_mine,
    minions::ice_dragon,
    schemes::dungeon_annex,
];

pub static CARDS: Lazy<HashMap<CardName, CardDefinition>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for card_fn in DEFINITIONS {
        let card = card_fn();
        map.insert(card.name, card);
    }
    map
});

pub fn get(name: CardName) -> &'static CardDefinition {
    CARDS.get(&name).expect("Card is not defined")
}
