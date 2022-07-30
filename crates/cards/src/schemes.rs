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

use card_helpers::{text, *};
use data::card_definition::{Ability, AbilityType, CardConfig, CardDefinition, SchemePoints};
use data::card_name::CardName;
use data::delegates::{Delegate, EventDelegate, QueryDelegate};
use data::primitives::{CardType, Rarity, School, Side};
use data::text::Keyword;
use display::rexard_images;
use display::rexard_images::RexardPack;
use rules::mutations::SummonMinion;
use rules::{mana, mutations, queries};

pub fn gold_mine() -> CardDefinition {
    CardDefinition {
        name: CardName::GoldMine,
        cost: scheme_cost(),
        image: rexard_images::get(RexardPack::MiningIcons, "MiningIcons_08_b"),
        card_type: CardType::Scheme,
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            text: text![Keyword::Score, "Gain", mana_text(7)],
            ability_type: AbilityType::Standard,
            delegates: vec![on_overlord_score(|g, s, _| {
                mana::gain(g, s.side(), 7);
                Ok(())
            })],
        }],
        config: CardConfig {
            stats: scheme_points(SchemePoints { level_requirement: 4, points: 2 }),
            ..CardConfig::default()
        },
    }
}

pub fn activate_reinforcements() -> CardDefinition {
    CardDefinition {
        name: CardName::ActivateReinforcements,
        cost: scheme_cost(),
        image: rexard_images::spell(1, "SpellBook01_01"),
        card_type: CardType::Scheme,
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            text: text![
                "When this scheme is scored by either player, summon a face down minion for free"
            ],
            ability_type: AbilityType::Standard,
            delegates: vec![Delegate::ScoreCard(EventDelegate {
                requirement: this_card,
                mutation: |g, _, _| {
                    if let Some(minion_id) =
                        queries::highest_cost(g.minions().filter(|c| c.is_face_down()))
                    {
                        mutations::summon_minion(g, minion_id, SummonMinion::IgnoreCosts)?;
                    }
                    Ok(())
                },
            })],
        }],
        config: CardConfig {
            stats: scheme_points(SchemePoints { level_requirement: 5, points: 3 }),
            ..CardConfig::default()
        },
    }
}

pub fn research_project() -> CardDefinition {
    CardDefinition {
        name: CardName::ResearchProject,
        cost: scheme_cost(),
        image: rexard_images::spell(1, "SpellBook01_03"),
        card_type: CardType::Scheme,
        side: Side::Overlord,
        school: School::Law,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            text: text![Keyword::Score, "Draw 2 cards.", "You get +2 maximum hand size."],
            ability_type: AbilityType::Standard,
            delegates: vec![
                on_overlord_score(|g, s, _| mutations::draw_cards(g, s.side(), 2).map(|_| ())),
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
