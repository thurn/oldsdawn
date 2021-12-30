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

//! All game rules, card definitions, and associated helpers

use std::collections::HashMap;

use data::card_definition::{CardConfig, CardDefinition, Cost};
use data::card_name::CardName;
use data::primitives::{CardType, Rarity, School, Side};
use once_cell::sync::Lazy;

pub mod abilities;
pub mod actions;
pub mod card_text;
pub mod champion_spells;
pub mod dispatch;
pub mod helpers;
pub mod minions;
pub mod mutations;
pub mod projects;
pub mod queries;
pub mod schemes;
pub mod weapons;

// TODO: Switch back to the linkme crate once https://github.com/dtolnay/linkme/issues/41 is fixed
static DEFINITIONS: &[fn() -> CardDefinition] = &[
    test_overlord_identity,
    test_champion_identity,
    test_overlord_spell,
    test_champion_spell,
    champion_spells::arcane_recovery,
    weapons::greataxe,
    projects::gold_mine,
    minions::ice_dragon,
    schemes::dungeon_annex,
];

/// Contains [CardDefinition]s for all known cards, keyed by [CardName]
static CARDS: Lazy<HashMap<CardName, CardDefinition>> = Lazy::new(|| {
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

fn test_overlord_identity() -> CardDefinition {
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

fn test_champion_identity() -> CardDefinition {
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

fn test_overlord_spell() -> CardDefinition {
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

fn test_champion_spell() -> CardDefinition {
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
