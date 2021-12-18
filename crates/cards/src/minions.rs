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

use crate::abilities;
use model::card_definition::{AbilityText, CardConfig, CardDefinition, CardStats, Keyword};
use model::card_name::CardName;
use model::primitives::{CardType, Faction, Rarity, School, Side};

use crate::card_helpers::*;

pub fn ice_dragon() -> CardDefinition {
    CardDefinition {
        name: CardName::IceDragon,
        cost: cost(8),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_44"),
        card_type: CardType::Minion,
        side: Side::Overlord,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![abilities::strike::<2>(), abilities::end_raid()],
        config: CardConfig {
            stats: health(5),
            faction: Some(Faction::Infernal),
            ..CardConfig::default()
        },
    }
}
