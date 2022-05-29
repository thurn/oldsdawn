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

//! Card definitions for the Minion card type

use data::card_definition::{Ability, AbilityType, CardConfig, CardDefinition, CardStats};
use data::card_name::CardName;
use data::card_state::CardPosition;
use data::game_actions::CardPromptAction;
use data::primitives::{CardType, ColdDamage, Faction, Rarity, RoomLocation, School, Side};
use data::text::Keyword;
use linkme::distributed_slice;
use rules::helpers::*;
use rules::mutations::SummonMinion;
use rules::{abilities, mutations, queries, text, DEFINITIONS};

pub fn initialize() {}

#[distributed_slice(DEFINITIONS)]
pub fn ice_dragon() -> CardDefinition {
    CardDefinition {
        name: CardName::IceDragon,
        cost: cost(3),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_44"),
        card_type: CardType::Minion,
        side: Side::Overlord,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![abilities::combat_deal_damage::<ColdDamage, 1>(), abilities::end_raid()],
        config: CardConfig {
            stats: CardStats { health: Some(5), shield: Some(1), ..CardStats::default() },
            faction: Some(Faction::Infernal),
            ..CardConfig::default()
        },
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn time_golem() -> CardDefinition {
    CardDefinition {
        name: CardName::TimeGolem,
        cost: cost(2),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_14"),
        card_type: CardType::Minion,
        side: Side::Overlord,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::construct(),
            Ability {
                text: text![
                    Keyword::Encounter,
                    "End the raid unless the Champion pays",
                    mana_text(5),
                    "or",
                    actions_text(2)
                ],
                ability_type: AbilityType::Standard,
                delegates: vec![on_encountered(|g, _s, _| {
                    set_card_prompt(
                        g,
                        Side::Champion,
                        vec![
                            Some(CardPromptAction::EndRaid),
                            lose_mana_prompt(g, Side::Champion, 5),
                            lose_actions_prompt(g, Side::Champion, 2),
                        ],
                    );
                })],
            },
        ],
        config: CardConfig {
            stats: health(3),
            faction: Some(Faction::Construct),
            ..CardConfig::default()
        },
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn temporal_vortex() -> CardDefinition {
    CardDefinition {
        name: CardName::TemporalVortex,
        cost: cost(6),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_15"),
        card_type: CardType::Minion,
        side: Side::Overlord,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![
            Ability {
                text: text![
                    Keyword::Combat,
                    "End the raid unless the Champion pays",
                    actions_text(2)
                ],
                ability_type: AbilityType::Standard,
                delegates: vec![minion_combat_actions(|g, _, _, _| {
                    vec![Some(CardPromptAction::EndRaid), lose_actions_prompt(g, Side::Champion, 2)]
                })],
            },
            Ability {
                text: text![
                    Keyword::Combat,
                    "Summon a minion from the Sanctum or Crypts for free.",
                ],
                ability_type: AbilityType::Standard,
                delegates: vec![combat(|g, s, _| {
                    let cards = g.hand(Side::Overlord).chain(g.discard_pile(Side::Overlord));
                    if let Some(minion_id) = queries::highest_cost(cards) {
                        let (room_id, index) =
                            queries::minion_position(g, s.card_id()).expect("position");
                        mutations::move_card(
                            g,
                            minion_id,
                            CardPosition::Room(room_id, RoomLocation::Defender),
                        );
                        g.move_card_to_index(minion_id, index);
                        mutations::summon_minion(g, minion_id, SummonMinion::IgnoreCosts);
                        mutations::set_raid_encountering_minion(g, s.card_id());
                    }
                })],
            },
        ],
        config: CardConfig {
            stats: CardStats { health: Some(6), shield: Some(3), ..CardStats::default() },
            faction: Some(Faction::Abyssal),
            ..CardConfig::default()
        },
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn shadow_lurker() -> CardDefinition {
    CardDefinition {
        name: CardName::ShadowLurker,
        cost: cost(3),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_16"),
        card_type: CardType::Minion,
        side: Side::Overlord,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![
            Ability {
                text: text!["While this minion is in an outer room, it has +2 health"],
                ability_type: AbilityType::Standard,
                delegates: vec![on_calculate_health(|g, s, _, current| {
                    match g.card(s.card_id()).position() {
                        CardPosition::Room(room_id, _) if !is_inner_room(room_id) => current + 2,
                        _ => current,
                    }
                })],
            },
            abilities::end_raid(),
        ],
        config: CardConfig {
            stats: CardStats { health: Some(2), shield: Some(1), ..CardStats::default() },
            faction: Some(Faction::Abyssal),
            ..CardConfig::default()
        },
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn sphinx_of_winters_breath() -> CardDefinition {
    CardDefinition {
        name: CardName::SphinxOfWintersBreath,
        cost: cost(2),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_44"),
        card_type: CardType::Minion,
        side: Side::Overlord,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            text: text![
                Keyword::Combat,
                "Deal 1 cold damage.",
                "If a card with an odd mana cost is discarded, end the raid."
            ],
            ability_type: AbilityType::Standard,
            delegates: vec![],
        }],
        config: CardConfig {
            stats: CardStats { health: Some(3), shield: Some(1), ..CardStats::default() },
            faction: Some(Faction::Mortal),
            ..CardConfig::default()
        },
    }
}
