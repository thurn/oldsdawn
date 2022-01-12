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

//! Card definitions for the Spell card type & Champion player

use data::card_definition::{CardConfig, CardDefinition};
use data::card_name::CardName;
use data::primitives::{CardType, Rarity, School, Side};
use linkme::distributed_slice;
use rules::card_text::text;
use rules::helpers::*;
use rules::{mutations, DEFINITIONS};

pub fn initialize() {}

#[distributed_slice(DEFINITIONS)]
pub fn arcane_recovery() -> CardDefinition {
    CardDefinition {
        name: CardName::ArcaneRecovery,
        cost: cost(5),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_24"),
        card_type: CardType::Spell,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![on_cast(text!("Gain", mana(9)), |g, s, _| {
            mutations::gain_mana(g, s.side(), 9)
        })],
        config: CardConfig::default(),
    }
}
