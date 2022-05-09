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

use data::card_definition::{Ability, AttackBoost, CardConfig, CardDefinition, Cost, SchemePoints};
use data::card_name::CardName;
use data::primitives::{
    CardType, ColdDamage, Faction, HealthValue, ManaValue, Rarity, School, Side,
};
use data::text::{Keyword, Sentence};
use linkme::distributed_slice;
use rules::helpers::*;
use rules::mutations::OnEmpty;
use rules::{abilities, mutations, text, DEFINITIONS};

pub const MINION_COST: ManaValue = 3;
pub const ARTIFACT_COST: ManaValue = 1;
pub const UNVEIL_COST: ManaValue = 3;
pub const MANA_STORED: ManaValue = 10;
pub const MANA_TAKEN: ManaValue = 2;
pub const MINION_HEALTH: HealthValue = 5;
pub const TEST_FACTION: Faction = Faction::Infernal;

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
        cost: actions(1),
        card_type: CardType::Scheme,
        config: CardConfig {
            stats: scheme_points(SchemePoints { level_requirement: 3, points: 1 }),
            ..CardConfig::default()
        },
        ..test_overlord_spell()
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn test_minion_end_raid() -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionEndRaid,
        cost: cost(MINION_COST),
        abilities: vec![abilities::end_raid()],
        card_type: CardType::Minion,
        config: CardConfig {
            stats: health(MINION_HEALTH),
            faction: Some(TEST_FACTION),
            ..CardConfig::default()
        },
        ..test_overlord_spell()
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn test_minion_deal_damage() -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionDealDamage,
        cost: cost(1),
        abilities: vec![abilities::deal_damage::<ColdDamage, 1>()],
        card_type: CardType::Minion,
        config: CardConfig {
            stats: health(MINION_HEALTH),
            faction: Some(TEST_FACTION),
            ..CardConfig::default()
        },
        ..test_overlord_spell()
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn test_minion_infernal() -> CardDefinition {
    CardDefinition {
        name: CardName::TestInfernalMinion,
        config: CardConfig {
            stats: health(MINION_HEALTH),
            faction: Some(TEST_FACTION),
            ..CardConfig::default()
        },
        ..test_minion_end_raid()
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn test_minion_abyssal() -> CardDefinition {
    CardDefinition {
        name: CardName::TestAbyssalMinion,
        config: CardConfig {
            stats: health(MINION_HEALTH),
            faction: Some(Faction::Abyssal),
            ..CardConfig::default()
        },
        ..test_minion_end_raid()
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn test_minion_mortal() -> CardDefinition {
    CardDefinition {
        name: CardName::TestMortalMinion,
        config: CardConfig {
            stats: health(MINION_HEALTH),
            faction: Some(Faction::Mortal),
            ..CardConfig::default()
        },
        ..test_minion_end_raid()
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
            faction: Some(TEST_FACTION),
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
            faction: Some(TEST_FACTION),
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
            faction: Some(TEST_FACTION),
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
            faction: Some(TEST_FACTION),
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
            faction: Some(TEST_FACTION),
            ..CardConfig::default()
        },
        ..test_weapon_2_attack()
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn activated_ability_take_mana() -> CardDefinition {
    CardDefinition {
        name: CardName::TestActivatedAbilityTakeMana,
        cost: cost(ARTIFACT_COST),
        card_type: CardType::Artifact,
        abilities: vec![
            abilities::store_mana_on_play::<MANA_STORED>(),
            abilities::activated_take_mana::<MANA_TAKEN>(actions(1)),
        ],
        config: CardConfig::default(),
        ..test_champion_spell()
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn triggered_ability_take_mana() -> CardDefinition {
    CardDefinition {
        name: CardName::TestTriggeredAbilityTakeManaAtDusk,
        cost: cost(UNVEIL_COST),
        card_type: CardType::Project,
        abilities: vec![
            abilities::unveil_at_dusk_then_store::<MANA_STORED>(),
            Ability {
                text: text![Keyword::Dusk, Keyword::Take(Sentence::Start, MANA_TAKEN)],
                ability_type: alert(),
                delegates: vec![at_dusk(|g, s, _| {
                    mutations::take_stored_mana(g, s.card_id(), MANA_TAKEN, OnEmpty::MoveToDiscard);
                })],
            },
        ],
        config: CardConfig::default(),
        ..test_overlord_spell()
    }
}
