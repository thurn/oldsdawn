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

use data::card_definition::{Ability, AbilityType, CardConfig, CardDefinition, TargetRequirement};
use data::card_name::CardName;
use data::delegates::{Delegate, QueryDelegate};
use data::primitives::{CardType, Rarity, RoomId, School, Side};
use rules::helpers::*;
use rules::text_macro::text;
use rules::{flags, mana, mutations};

pub fn arcane_recovery() -> CardDefinition {
    CardDefinition {
        name: CardName::ArcaneRecovery,
        cost: cost(5),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_25"),
        card_type: CardType::ChampionSpell,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![simple_ability(
            text!("Gain", mana_text(9)),
            on_cast(|g, s, _| {
                mana::gain(g, s.side(), 9);
                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn meditation() -> CardDefinition {
    CardDefinition {
        name: CardName::Meditation,
        cost: cost(1),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_24"),
        card_type: CardType::ChampionSpell,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![simple_ability(
            text!("Gain", mana_text(5), ".", "Lose", actions_text(1), reminder("(if able).")),
            on_cast(|g, s, _| {
                mana::gain(g, s.side(), 5);
                mutations::lose_action_points_if_able(g, s.side(), 1);
                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn coup_de_grace() -> CardDefinition {
    CardDefinition {
        name: CardName::CoupDeGrace,
        cost: cost(0),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_26"),
        card_type: CardType::ChampionSpell,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            text: text!(
                "Raid the Sanctum or Vault, accessing 1 additional card.",
                "If successful, draw a card."
            ),
            ability_type: AbilityType::Standard,
            delegates: vec![
                on_cast(|g, s, play_card| initiate_raid(g, s, play_card.target)),
                add_vault_access::<1>(matching_raid),
                add_sanctum_access::<1>(matching_raid),
                on_raid_success(matching_raid, |g, s, _| {
                    mutations::draw_cards(g, s.side(), 1).map(|_| ())
                }),
            ],
        }],
        config: CardConfig {
            custom_targeting: Some(TargetRequirement::TargetRoom(|game, _, room_id| {
                flags::can_take_initiate_raid_action(game, Side::Champion, room_id)
                    && (room_id == RoomId::Sanctum || room_id == RoomId::Vault)
            })),
            ..CardConfig::default()
        },
    }
}

pub fn charged_strike() -> CardDefinition {
    CardDefinition {
        name: CardName::ChargedStrike,
        cost: cost(1),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_27"),
        card_type: CardType::ChampionSpell,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![simple_ability(
            text!("Initiate a raid.", "Gain", mana_text(5), "to spend during that raid."),
            on_cast(|g, s, play_card| {
                initiate_raid_with_callback(g, s, play_card.target, |game, raid_id| {
                    mana::add_raid_specific_mana(game, s.side(), raid_id, 5);
                })
            }),
        )],
        config: CardConfig {
            custom_targeting: Some(TargetRequirement::TargetRoom(|game, _, room_id| {
                flags::can_take_initiate_raid_action(game, Side::Champion, room_id)
            })),
            ..CardConfig::default()
        },
    }
}

pub fn stealth_mission() -> CardDefinition {
    CardDefinition {
        name: CardName::StealthMission,
        cost: cost(1),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_28"),
        card_type: CardType::ChampionSpell,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            text: text!(
                "Initiate a raid.",
                "During that raid, summon costs are increased by",
                mana_text(3),
                "."
            ),
            ability_type: AbilityType::Standard,
            delegates: vec![
                on_cast(|g, s, play_card| initiate_raid(g, s, play_card.target)),
                Delegate::ManaCost(QueryDelegate {
                    requirement: matching_raid,
                    transformation: |g, _s, card_id, current| {
                        if rules::card_definition(g, *card_id).card_type == CardType::Minion {
                            current.map(|current| current + 3)
                        } else {
                            current
                        }
                    },
                }),
            ],
        }],
        config: CardConfig {
            custom_targeting: Some(TargetRequirement::TargetRoom(|game, _, room_id| {
                flags::can_take_initiate_raid_action(game, Side::Champion, room_id)
            })),
            ..CardConfig::default()
        },
    }
}

pub fn preparation() -> CardDefinition {
    CardDefinition {
        name: CardName::Preparation,
        cost: cost(1),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_29"),
        card_type: CardType::ChampionSpell,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![simple_ability(
            text!("Draw 4 cards.", "Lose", actions_text(1), reminder("(if able).")),
            on_cast(|g, s, _| {
                mutations::draw_cards(g, s.side(), 4)?;
                mutations::lose_action_points_if_able(g, s.side(), 1);
                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}
