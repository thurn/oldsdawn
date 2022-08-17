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

use card_helpers::{abilities, text, *};
use data::card_definition::{
    Ability, AbilityType, AttackBoost, CardConfig, CardDefinition, CardStats, SchemePoints,
    SpecialEffects,
};
use data::card_name::CardName;
use data::primitives::{CardType, HealthValue, Lineage, ManaValue, Rarity, School, Side, Sprite};
use data::special_effects::{Projectile, TimedEffect};
use data::text::{Keyword, Sentence};
use rules::mutations;
use rules::mutations::OnZeroStored;

pub const MINION_COST: ManaValue = 3;
pub const WEAPON_COST: ManaValue = 3;
pub const ARTIFACT_COST: ManaValue = 1;
pub const UNVEIL_COST: ManaValue = 3;
pub const MANA_STORED: ManaValue = 10;
pub const MANA_TAKEN: ManaValue = 2;
pub const MINION_HEALTH: HealthValue = 5;
pub const TEST_LINEAGE: Lineage = Lineage::Infernal;

pub fn test_overlord_identity() -> CardDefinition {
    CardDefinition {
        name: CardName::TestOverlordIdentity,
        cost: identity_cost(),
        image: Sprite::new("Enixion/Fantasy Art Pack 2/Resized/3.png"),
        card_type: CardType::Identity,
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::None,
        abilities: vec![],
        config: CardConfig::default(),
    }
}

pub fn test_champion_identity() -> CardDefinition {
    CardDefinition {
        name: CardName::TestChampionIdentity,
        cost: identity_cost(),
        image: Sprite::new("Enixion/Fantasy Art Pack 2/Resized/2.png"),
        card_type: CardType::Identity,
        side: Side::Champion,
        school: School::Primal,
        rarity: Rarity::None,
        abilities: vec![],
        config: CardConfig::default(),
    }
}

pub fn test_overlord_spell() -> CardDefinition {
    CardDefinition {
        name: CardName::TestOverlordSpell,
        cost: cost(1),
        card_type: CardType::OverlordSpell,
        ..test_overlord_identity()
    }
}

pub fn test_champion_spell() -> CardDefinition {
    CardDefinition {
        name: CardName::TestChampionSpell,
        cost: cost(1),
        card_type: CardType::ChampionSpell,
        ..test_champion_identity()
    }
}

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

pub fn test_project_2_cost() -> CardDefinition {
    CardDefinition {
        name: CardName::TestProject2Cost,
        cost: cost(2),
        card_type: CardType::Project,
        config: CardConfig::default(),
        ..test_overlord_spell()
    }
}

pub fn test_minion_end_raid() -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionEndRaid,
        cost: cost(MINION_COST),
        abilities: vec![abilities::end_raid()],
        card_type: CardType::Minion,
        config: CardConfig {
            stats: health(MINION_HEALTH),
            lineage: Some(TEST_LINEAGE),
            ..CardConfig::default()
        },
        ..test_overlord_spell()
    }
}

pub fn test_minion_shield_1() -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionShield1Infernal,
        cost: cost(MINION_COST),
        abilities: vec![abilities::end_raid()],
        card_type: CardType::Minion,
        config: CardConfig {
            stats: CardStats {
                health: Some(MINION_HEALTH),
                shield: Some(1),
                ..CardStats::default()
            },
            lineage: Some(TEST_LINEAGE),
            ..CardConfig::default()
        },
        ..test_overlord_spell()
    }
}

pub fn test_minion_shield_2_abyssal() -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionShield2Abyssal,
        cost: cost(MINION_COST),
        abilities: vec![abilities::end_raid()],
        card_type: CardType::Minion,
        config: CardConfig {
            stats: CardStats {
                health: Some(MINION_HEALTH),
                shield: Some(2),
                ..CardStats::default()
            },
            lineage: Some(Lineage::Abyssal),
            ..CardConfig::default()
        },
        ..test_overlord_spell()
    }
}

pub fn test_minion_deal_damage() -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionDealDamage,
        cost: cost(1),
        abilities: vec![abilities::combat_deal_damage::<1>()],
        card_type: CardType::Minion,
        config: CardConfig {
            stats: health(MINION_HEALTH),
            lineage: Some(TEST_LINEAGE),
            ..CardConfig::default()
        },
        ..test_overlord_spell()
    }
}

pub fn test_minion_infernal() -> CardDefinition {
    CardDefinition {
        name: CardName::TestInfernalMinion,
        config: CardConfig {
            stats: health(MINION_HEALTH),
            lineage: Some(TEST_LINEAGE),
            ..CardConfig::default()
        },
        ..test_minion_end_raid()
    }
}

pub fn test_minion_abyssal() -> CardDefinition {
    CardDefinition {
        name: CardName::TestAbyssalMinion,
        config: CardConfig {
            stats: health(MINION_HEALTH),
            lineage: Some(Lineage::Abyssal),
            ..CardConfig::default()
        },
        ..test_minion_end_raid()
    }
}

pub fn test_minion_mortal() -> CardDefinition {
    CardDefinition {
        name: CardName::TestMortalMinion,
        config: CardConfig {
            stats: health(MINION_HEALTH),
            lineage: Some(Lineage::Mortal),
            ..CardConfig::default()
        },
        ..test_minion_end_raid()
    }
}

pub fn test_weapon_2_attack() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeapon2Attack,
        cost: cost(WEAPON_COST),
        card_type: CardType::Weapon,
        config: CardConfig {
            stats: base_attack(2),
            lineage: Some(TEST_LINEAGE),
            ..CardConfig::default()
        },
        ..test_champion_spell()
    }
}

pub fn test_weapon_2_attack_12_boost() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeapon2Attack12Boost,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfig {
            stats: attack(2, AttackBoost { cost: 1, bonus: 2 }),
            lineage: Some(TEST_LINEAGE),
            ..CardConfig::default()
        },
        ..test_weapon_2_attack()
    }
}

pub fn test_weapon_3_attack_12_boost() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeapon3Attack12Boost3Cost,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfig {
            stats: attack(3, AttackBoost { cost: 1, bonus: 2 }),
            lineage: Some(TEST_LINEAGE),
            ..CardConfig::default()
        },
        ..test_weapon_2_attack()
    }
}

pub fn test_weapon_4_attack_12_boost() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeapon4Attack12Boost,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfig {
            stats: attack(4, AttackBoost { cost: 1, bonus: 2 }),
            lineage: Some(TEST_LINEAGE),
            ..CardConfig::default()
        },
        ..test_weapon_2_attack()
    }
}

pub fn test_weapon_abyssal() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeaponAbyssal,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfig {
            stats: attack(3, AttackBoost { cost: 1, bonus: 2 }),
            lineage: Some(Lineage::Abyssal),
            ..CardConfig::default()
        },
        ..test_weapon_2_attack()
    }
}

pub fn test_weapon_infernal() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeaponInfernal,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfig {
            stats: attack(3, AttackBoost { cost: 1, bonus: 2 }),
            lineage: Some(Lineage::Infernal),
            ..CardConfig::default()
        },
        ..test_weapon_2_attack()
    }
}

pub fn test_weapon_mortal() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeaponMortal,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfig {
            stats: attack(3, AttackBoost { cost: 1, bonus: 2 }),
            lineage: Some(Lineage::Mortal),
            ..CardConfig::default()
        },
        ..test_weapon_2_attack()
    }
}

pub fn test_weapon_5_attack() -> CardDefinition {
    CardDefinition {
        name: CardName::TestWeapon5Attack,
        config: CardConfig {
            stats: base_attack(5),
            lineage: Some(TEST_LINEAGE),
            ..CardConfig::default()
        },
        ..test_weapon_2_attack()
    }
}

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

pub fn triggered_ability_take_mana() -> CardDefinition {
    CardDefinition {
        name: CardName::TestTriggeredAbilityTakeManaAtDusk,
        cost: cost(UNVEIL_COST),
        card_type: CardType::Project,
        abilities: vec![
            Ability {
                text: text![
                    Keyword::Unveil,
                    "this project at Dusk, then",
                    Keyword::Store(Sentence::Start, MANA_STORED)
                ],
                ability_type: AbilityType::Standard,
                delegates: vec![unveil_at_dusk(), store_mana_on_unveil::<MANA_STORED>()],
            },
            Ability {
                text: text![Keyword::Dusk, Keyword::Take(Sentence::Start, MANA_TAKEN)],
                ability_type: AbilityType::Standard,
                delegates: vec![at_dusk(|g, s, _| {
                    mutations::take_stored_mana(
                        g,
                        s.card_id(),
                        MANA_TAKEN,
                        OnZeroStored::Sacrifice,
                    )?;
                    alert(g, s);
                    Ok(())
                })],
            },
        ],
        config: CardConfig::default(),
        ..test_overlord_spell()
    }
}

pub fn test_0_cost_champion_spell() -> CardDefinition {
    CardDefinition {
        name: CardName::Test0CostChampionSpell,
        cost: cost(0),
        ..test_champion_spell()
    }
}

pub fn test_1_cost_champion_spell() -> CardDefinition {
    CardDefinition {
        name: CardName::Test1CostChampionSpell,
        cost: cost(1),
        ..test_champion_spell()
    }
}

pub fn deal_damage_end_raid() -> CardDefinition {
    CardDefinition {
        name: CardName::TestMinionDealDamageEndRaid,
        cost: cost(3),
        card_type: CardType::Minion,
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![abilities::combat_deal_damage::<1>(), abilities::end_raid()],
        config: CardConfig {
            stats: CardStats { health: Some(5), shield: Some(1), ..CardStats::default() },
            lineage: Some(Lineage::Infernal),
            ..CardConfig::default()
        },
        ..test_overlord_spell()
    }
}

pub fn test_card_stored_mana() -> CardDefinition {
    CardDefinition {
        name: CardName::TestCardStoredMana,
        cost: cost(4),
        card_type: CardType::Project,
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            Ability {
                text: text![Keyword::Unveil, "at Dusk, then", Keyword::Store(Sentence::Start, 12)],
                ability_type: AbilityType::Standard,
                delegates: vec![unveil_at_dusk(), store_mana_on_unveil::<12>()],
            },
            simple_ability(
                text![Keyword::Dusk, Keyword::Take(Sentence::Start, 3)],
                at_dusk(|g, s, _| {
                    mutations::take_stored_mana(g, s.card_id(), 3, OnZeroStored::Sacrifice)?;
                    alert(g, s);
                    Ok(())
                }),
            ),
        ],
        config: CardConfig::default(),
        ..test_overlord_spell()
    }
}

pub fn test_attack_weapon() -> CardDefinition {
    CardDefinition {
        name: CardName::TestAttackWeapon,
        cost: cost(3),
        card_type: CardType::Weapon,
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfig {
            stats: attack(3, AttackBoost { cost: 1, bonus: 2 }),
            lineage: Some(Lineage::Infernal),
            special_effects: SpecialEffects {
                projectile: Some(Projectile::Hovl(8)),
                additional_hit: Some(TimedEffect::HovlSwordSlash(1)),
            },
            ..CardConfig::default()
        },
        ..test_champion_spell()
    }
}
