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

use card_helpers::{abilities, text, *};
use data::card_definition::{
    Ability, AbilityType, AttackBoost, CardConfig, CardDefinition, CardStats, SpecialEffects,
};
use data::card_name::CardName;
use data::delegates::{Delegate, QueryDelegate};
use data::primitives::{CardType, Lineage, Rarity, School, Side};
use data::special_effects::{Projectile, TimedEffect};
use data::text::Keyword;
use data::utils;
use display::rexard_images;
use display::rexard_images::RexardWeaponType;
use rules::mutations;

pub fn marauders_axe() -> CardDefinition {
    CardDefinition {
        name: CardName::MaraudersAxe,
        cost: cost(5),
        image: rexard_images::weapon(RexardWeaponType::Axes, "a_n_b_01"),
        card_type: CardType::Weapon,
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            Ability {
                text: text![
                    Keyword::SuccessfulRaid,
                    "This weapon costs",
                    mana_text(2),
                    "less to play this turn."
                ],
                ability_type: AbilityType::Standard,
                delegates: vec![
                    on_raid_success(always, |g, s, _| {
                        save_turn(g, s);
                        Ok(())
                    }),
                    Delegate::ManaCost(QueryDelegate {
                        requirement: this_card,
                        transformation: |g, s, _, value| {
                            if utils::is_true(|| Some(g.ability_state(s)?.turn? == g.data.turn)) {
                                value.map(|v| v.saturating_sub(2))
                            } else {
                                value
                            }
                        },
                    }),
                ],
            },
            abilities::encounter_boost(),
        ],
        config: CardConfig {
            stats: attack(2, AttackBoost { cost: 2, bonus: 3 }),
            lineage: Some(Lineage::Infernal),
            special_effects: SpecialEffects {
                projectile: Some(Projectile::Hovl(8)),
                additional_hit: Some(TimedEffect::HovlSwordSlash(1)),
            },
            ..CardConfig::default()
        },
    }
}

pub fn keen_halberd() -> CardDefinition {
    CardDefinition {
        name: CardName::KeenHalberd,
        cost: cost(3),
        image: rexard_images::weapon(RexardWeaponType::Polearms, "sp_b_04"),
        card_type: CardType::Weapon,
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![abilities::encounter_boost()],
        config: CardConfig {
            stats: CardStats {
                base_attack: Some(3),
                attack_boost: Some(AttackBoost { cost: 2, bonus: 1 }),
                breach: Some(1),
                ..CardStats::default()
            },
            lineage: Some(Lineage::Abyssal),
            special_effects: projectile(Projectile::Hovl(2)),
            ..CardConfig::default()
        },
    }
}

pub fn ethereal_blade() -> CardDefinition {
    CardDefinition {
        name: CardName::EtherealBlade,
        cost: cost(1),
        image: rexard_images::weapon(RexardWeaponType::Swords, "sv_b_01"),
        card_type: CardType::Weapon,
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::encounter_boost(),
            Ability {
                text: text!["When you use this weapon, sacrifice it at the end of the raid."],
                ability_type: AbilityType::Standard,
                delegates: vec![
                    on_weapon_used(
                        |_g, s, used_weapon| used_weapon.weapon_id == s.card_id(),
                        |g, s, used_weapon| save_raid_id(g, s, &used_weapon.raid_id),
                    ),
                    on_raid_ended(matching_raid, |g, s, _| {
                        mutations::sacrifice_card(g, s.card_id())?;
                        alert(g, s);
                        Ok(())
                    }),
                ],
            },
        ],
        config: CardConfig {
            stats: attack(1, AttackBoost { cost: 1, bonus: 1 }),
            lineage: Some(Lineage::Prismatic),
            special_effects: projectile(Projectile::Hovl(3)),
            ..CardConfig::default()
        },
    }
}

pub fn bow_of_the_alliance() -> CardDefinition {
    CardDefinition {
        name: CardName::BowOfTheAlliance,
        cost: cost(3),
        image: rexard_images::weapon(RexardWeaponType::Bows, "b_b_01"),
        card_type: CardType::Weapon,
        side: Side::Champion,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::encounter_boost(),
            simple_ability(
                text!["+1 attack per weapon you control"],
                Delegate::AttackBoost(QueryDelegate {
                    requirement: this_card,
                    transformation: |g, _s, _, boost| AttackBoost {
                        bonus: g.weapons().count() as u32,
                        ..boost
                    },
                }),
            ),
        ],
        config: CardConfig {
            stats: attack(1, AttackBoost { cost: 1, bonus: 0 }),
            lineage: Some(Lineage::Mortal),
            special_effects: projectile(Projectile::Hovl(4)),
            ..CardConfig::default()
        },
    }
}
