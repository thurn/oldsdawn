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

use data::card_definition::{CardConfig, CardDefinition, Cost};
use data::card_name::CardName;
use data::primitives::{CardType, Rarity, School, Side};
use linkme::distributed_slice;
use rules::{helpers, DEFINITIONS};

pub fn initialize() {}

#[distributed_slice(DEFINITIONS)]
pub fn test_overlord_identity() -> CardDefinition {
    CardDefinition {
        name: CardName::TestOverlordIdentity,
        cost: Cost { mana: None, actions: 0 },
        image: helpers::sprite("Enixion/Fantasy Art Pack 2/Resized/3"),
        card_type: CardType::Identity,
        side: Side::Overlord,
        school: School::Time,
        rarity: Rarity::None,
        abilities: vec![],
        config: CardConfig::default(),
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn test_champion_identity() -> CardDefinition {
    CardDefinition {
        name: CardName::TestChampionIdentity,
        cost: Cost { mana: None, actions: 0 },
        image: helpers::sprite("Enixion/Fantasy Art Pack 2/Resized/2"),
        card_type: CardType::Identity,
        side: Side::Champion,
        school: School::Nature,
        rarity: Rarity::None,
        abilities: vec![],
        config: CardConfig::default(),
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn test_overlord_spell() -> CardDefinition {
    CardDefinition {
        name: CardName::TestOverlordSpell,
        cost: Cost { mana: None, actions: 0 },
        image: helpers::sprite("Enixion/Fantasy Art Pack 2/Resized/3"),
        card_type: CardType::Spell,
        side: Side::Overlord,
        school: School::Time,
        rarity: Rarity::None,
        abilities: vec![],
        config: CardConfig::default(),
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn test_champion_spell() -> CardDefinition {
    CardDefinition {
        name: CardName::TestChampionSpell,
        cost: Cost { mana: None, actions: 0 },
        image: helpers::sprite("Enixion/Fantasy Art Pack 2/Resized/2"),
        card_type: CardType::Spell,
        side: Side::Champion,
        school: School::Nature,
        rarity: Rarity::None,
        abilities: vec![],
        config: CardConfig::default(),
    }
}
