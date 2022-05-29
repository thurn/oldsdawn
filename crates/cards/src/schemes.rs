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

//! Card definitions for the Scheme card type

use data::card_definition::{Ability, AbilityType, CardConfig, CardDefinition, SchemePoints};
use data::card_name::CardName;
use data::delegates::{Delegate, EventDelegate, QueryDelegate};
use data::primitives::{CardType, Rarity, School, Side};
use data::text::Keyword;
use linkme::distributed_slice;
use rules::helpers::*;
use rules::mutations::SummonMinion;
use rules::text_macro::text;
use rules::{mana, mutations, queries, DEFINITIONS};

pub fn initialize() {}

#[distributed_slice(DEFINITIONS)]
pub fn dungeon_annex() -> CardDefinition {
    CardDefinition {
        name: CardName::DungeonAnnex,
        cost: scheme_cost(),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_45"),
        card_type: CardType::Scheme,
        side: Side::Overlord,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            text: text![Keyword::Score, "Gain", mana_text(7)],
            ability_type: AbilityType::Standard,
            delegates: vec![on_overlord_score(|g, s, _| {
                mana::gain(g, s.side(), 7);
            })],
        }],
        config: CardConfig {
            stats: scheme_points(SchemePoints { level_requirement: 4, points: 2 }),
            ..CardConfig::default()
        },
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn activate_reinforcements() -> CardDefinition {
    CardDefinition {
        name: CardName::ActivateReinforcements,
        cost: scheme_cost(),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_15"),
        card_type: CardType::Scheme,
        side: Side::Overlord,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            text: text![
                "When this scheme is scored by either player, summon a face down minion for free"
            ],
            ability_type: AbilityType::Standard,
            delegates: vec![Delegate::ScoreCard(EventDelegate {
                requirement: this_card,
                mutation: |g, s, _| {
                    if let Some(minion_id) =
                        queries::highest_cost(g.minions().filter(|c| c.is_face_down()))
                    {
                        mutations::summon_minion(g, minion_id, SummonMinion::IgnoreCosts);
                        alert(g, s);
                    }
                },
            })],
        }],
        config: CardConfig {
            stats: scheme_points(SchemePoints { level_requirement: 5, points: 3 }),
            ..CardConfig::default()
        },
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn research_project() -> CardDefinition {
    CardDefinition {
        name: CardName::ResearchProject,
        cost: scheme_cost(),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_16"),
        card_type: CardType::Scheme,
        side: Side::Overlord,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            text: text![Keyword::Score, "Draw 2 cards.", "You get +2 maximum hand size."],
            ability_type: AbilityType::Standard,
            delegates: vec![
                on_overlord_score(|g, s, _| {
                    mutations::draw_cards(g, s.side(), 2);
                }),
                Delegate::MaximumHandSize(QueryDelegate {
                    requirement: scored_by_owner,
                    transformation: |_, s, side, current| {
                        if s.side() == *side {
                            current + 2
                        } else {
                            current
                        }
                    },
                }),
            ],
        }],
        config: CardConfig {
            stats: scheme_points(SchemePoints { level_requirement: 3, points: 1 }),
            ..CardConfig::default()
        },
    }
}
