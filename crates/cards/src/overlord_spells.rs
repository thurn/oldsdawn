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

//! Card definitions for the Spell card type & Overlord player

use data::card_definition::{Ability, AbilityType, CardConfig, CardDefinition};
use data::card_name::CardName;
use data::primitives::{CardType, Rarity, School, Side};
use linkme::distributed_slice;
use rules::helpers::*;
use rules::text_macro::text;
use rules::{mana, DEFINITIONS};

pub fn initialize() {}

#[distributed_slice(DEFINITIONS)]
pub fn gathering_dark() -> CardDefinition {
    CardDefinition {
        name: CardName::GatheringDark,
        cost: cost(5),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_45"),
        card_type: CardType::OverlordSpell,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            text: text!("Gain", mana_text(9)),
            ability_type: AbilityType::Standard,
            delegates: vec![on_cast(|g, s, _| mana::gain(g, s.side(), 9))],
        }],
        config: CardConfig::default(),
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn overwhelming_power() -> CardDefinition {
    CardDefinition {
        name: CardName::OverwhelmingPower,
        cost: cost(10),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_46"),
        card_type: CardType::OverlordSpell,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            text: text!("Gain", mana_text(15)),
            ability_type: AbilityType::Standard,
            delegates: vec![on_cast(|g, s, _| mana::gain(g, s.side(), 15))],
        }],
        config: CardConfig::default(),
    }
}
