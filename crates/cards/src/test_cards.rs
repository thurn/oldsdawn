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

use data::card_definition::{AttackBoost, CardConfig, CardDefinition, Cost, SchemePoints};
use data::card_name::CardName;
use data::primitives::{CardType, ColdDamage, Faction, Rarity, School, Side};
use linkme::distributed_slice;
use rules::helpers::*;
use rules::{abilities, DEFINITIONS};

pub fn initialize() {}

#[distributed_slice(DEFINITIONS)]
pub fn test_overlord_identity() -> CardDefinition {
    CardDefinition {
        name: CardName::TestOverlordIdentity,
        cost: Cost { mana: None, actions: 0 },
        image: sprite("Enixion/Fantasy Art Pack 2/Resized/3"),
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
        image: sprite("Enixion/Fantasy Art Pack 2/Resized/2"),
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
        cost: cost(1),
        card_type: CardType::Sorcery,
        ..test_overlord_identity()
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn test_champion_spell() -> CardDefinition {
    CardDefinition {
        name: CardName::TestChampionSpell,
        cost: cost(1),
        card_type: CardType::Spell,
        ..test_champion_identity()
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn test_scheme_31() -> CardDefinition {
    CardDefinition {
        name: CardName::TestScheme31,
        cost: scheme_cost(),
        card_type: CardType::Scheme,
        config: CardConfig {
            stats: scheme_points(SchemePoints { level_requirement: 3, points: 1 }),
            ..CardConfig::default()
        },
        ..test_overlord_spell()
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn test_minion_alpha() -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionAlpha,
        cost: cost(3),
        abilities: vec![abilities::end_raid()],
        card_type: CardType::Minion,
        config: CardConfig {
            stats: health(5),
            faction: Some(Faction::Infernal),
            ..CardConfig::default()
        },
        ..test_overlord_spell()
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn test_minion_beta() -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionBeta,
        cost: cost(1),
        abilities: vec![abilities::deal_damage::<ColdDamage, 1>()],
        card_type: CardType::Minion,
        config: CardConfig {
            stats: health(5),
            faction: Some(Faction::Infernal),
            ..CardConfig::default()
        },
        ..test_overlord_spell()
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn test_weapon_2_attack() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeapon2Attack,
        cost: cost(3),
        card_type: CardType::Weapon,
        config: CardConfig {
            stats: base_attack(2),
            faction: Some(Faction::Infernal),
            ..CardConfig::default()
        },
        ..test_champion_spell()
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn test_weapon_2_attack_12_boost() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeapon2Attack12Boost,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfig {
            stats: attack(2, AttackBoost { cost: 1, bonus: 2 }),
            faction: Some(Faction::Infernal),
            ..CardConfig::default()
        },
        ..test_weapon_2_attack()
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn test_weapon_3_attack_12_boost() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeapon3Attack12Boost3Cost,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfig {
            stats: attack(3, AttackBoost { cost: 1, bonus: 2 }),
            faction: Some(Faction::Infernal),
            ..CardConfig::default()
        },
        ..test_weapon_2_attack()
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn test_weapon_4_attack_12_boost() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeapon4Attack12Boost,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfig {
            stats: attack(4, AttackBoost { cost: 1, bonus: 2 }),
            faction: Some(Faction::Infernal),
            ..CardConfig::default()
        },
        ..test_weapon_2_attack()
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn test_weapon_5_attack() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeapon5Attack,
        config: CardConfig {
            stats: base_attack(5),
            faction: Some(Faction::Infernal),
            ..CardConfig::default()
        },
        ..test_weapon_2_attack()
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn activated_ability_take_mana() -> CardDefinition {
    CardDefinition {
        name: CardName::TestActivatedAbilityTake1Mana,
        cost: cost(1),
        card_type: CardType::Artifact,
        abilities: vec![abilities::store_mana::<10>(), abilities::take_mana::<1>(cost(0))],
        config: CardConfig::default(),
        ..test_champion_spell()
    }
}
