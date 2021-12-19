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

use data::card_definition::{AbilityText, CardConfig, CardDefinition, Keyword};
use std::sync::Arc;

use data::card_name::CardName;
use data::primitives::{CardType, Rarity, School, Side};

use crate::card_helpers::*;

pub fn arcane_recovery() -> CardDefinition {
    CardDefinition {
        name: CardName::ArcaneRecovery,
        cost: cost(5),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_25"),
        card_type: CardType::Spell,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![on_play(
            AbilityText::Text(vec![text("Gain"), mana_symbol(9)]),
            |g, s, _| gain_mana(g, s.side(), 9),
        )],
        config: CardConfig::default(),
    }
}
