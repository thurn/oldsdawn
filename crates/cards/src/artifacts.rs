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

//! Card definitions for the Weapon card type

use data::card_definition::{CardConfig, CardDefinition};
use data::card_name::CardName;
use data::primitives::{CardType, Rarity, School, Side};
use linkme::distributed_slice;
use rules::helpers::*;
use rules::{abilities, DEFINITIONS};

pub fn initialize() {}

#[distributed_slice(DEFINITIONS)]
pub fn lodestone() -> CardDefinition {
    CardDefinition {
        name: CardName::Lodestone,
        cost: cost(1),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_78"),
        card_type: CardType::Artifact,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::store_mana::<12>(),
            abilities::activated_take_mana::<2>(cost_1_action()),
        ],
        config: CardConfig::default(),
    }
}
