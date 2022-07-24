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

use card_helpers::text_macro::text;
use card_helpers::{abilities, *};
use data::card_definition::{
    Ability, AbilityType, CardConfig, CardDefinition, Cost, TargetRequirement,
};
use data::card_name::CardName;
use data::delegates::{Delegate, EventDelegate};
use data::primitives::{CardType, Rarity, School, Side};
use data::text::{Keyword, Sentence};
use data::utils;
use display::rexard_images;
use display::rexard_images::{RexardArtifactType, RexardPack};
use rules::mutations;
use rules::mutations::OnZeroStored;

pub fn lodestone() -> CardDefinition {
    CardDefinition {
        name: CardName::Lodestone,
        cost: cost(1),
        image: rexard_images::get(RexardPack::MagicItems, "orb_04_b"),
        card_type: CardType::Artifact,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::store_mana_on_play::<12>(),
            abilities::activated_take_mana::<2>(actions(1)),
        ],
        config: CardConfig::default(),
    }
}

pub fn invisibility_ring() -> CardDefinition {
    CardDefinition {
        name: CardName::InvisibilityRing,
        cost: cost(2),
        image: rexard_images::get(RexardPack::JeweleryRings, "rn_b_03"),
        card_type: CardType::Artifact,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![Ability {
            text: text!(
                "The first time each turn you access the Sanctum, access 1 additional card."
            ),
            ability_type: AbilityType::Standard,
            delegates: vec![
                on_raid_access_start(face_up_in_play, |g, s, raid_id| {
                    once_per_turn(g, s, raid_id, save_raid_id)
                }),
                add_sanctum_access::<1>(matching_raid),
            ],
        }],
        config: CardConfig::default(),
    }
}

pub fn accumulator() -> CardDefinition {
    CardDefinition {
        name: CardName::Accumulator,
        cost: cost(3),
        image: rexard_images::get(RexardPack::JeweleryNecklaces, "07_ob"),
        card_type: CardType::Artifact,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![
            simple_ability(
                text!(Keyword::SuccessfulRaid, Keyword::Store(Sentence::Start, 1)),
                on_raid_success(face_up_in_play, |g, s, _| {
                    add_stored_mana(g, s.card_id(), 1);
                    alert(g, s);
                    Ok(())
                }),
            ),
            Ability {
                text: text!(Keyword::Store(Sentence::Start, 1), ", then take all stored mana."),
                ability_type: AbilityType::Activated(actions(1), TargetRequirement::None),
                delegates: vec![on_activated(|g, s, activated| {
                    let mana = add_stored_mana(g, s.card_id(), 1);
                    mutations::take_stored_mana(g, activated.card_id(), mana, OnZeroStored::Ignore)
                        .map(|_| ())
                })],
            },
        ],
        config: CardConfig::default(),
    }
}

pub fn mage_gloves() -> CardDefinition {
    CardDefinition {
        name: CardName::MageGloves,
        cost: cost(5),
        image: rexard_images::artifact(RexardArtifactType::Gloves, "gloves_20"),
        card_type: CardType::Artifact,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::store_mana_on_play::<12>(),
            Ability {
                text: text!(
                    "Raid an",
                    Keyword::InnerRoom(Sentence::Internal),
                    "you have not raided this turn.",
                    "If successful,",
                    Keyword::Take(Sentence::Internal, 3)
                ),
                ability_type: AbilityType::Activated(
                    actions(1),
                    TargetRequirement::TargetRoom(|g, _, room_id| {
                        is_inner_room(room_id)
                            && utils::is_false(|| {
                                Some(g.room_state.get(&room_id)?.last_raided? == g.data.turn)
                            })
                    }),
                ),
                delegates: vec![
                    on_activated(|g, s, activated| initiate_raid(g, s, activated.target)),
                    on_raid_success(matching_raid, |g, s, _| {
                        mutations::take_stored_mana(g, s.card_id(), 3, OnZeroStored::Sacrifice)
                            .map(|_| ())
                    }),
                ],
            },
        ],
        config: CardConfig::default(),
    }
}

pub fn skys_reach() -> CardDefinition {
    CardDefinition {
        name: CardName::SkysReach,
        cost: cost(0),
        image: rexard_images::artifact(RexardArtifactType::Belts, "belts_11"),
        card_type: CardType::Artifact,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![
            simple_ability(
                text![Keyword::Dawn, Keyword::Take(Sentence::Start, 1)],
                at_dawn(|g, s, _| {
                    let taken =
                        mutations::take_stored_mana(g, s.card_id(), 1, OnZeroStored::Ignore)?;
                    alert_if_nonzero(g, s, taken);
                    Ok(())
                }),
            ),
            Ability {
                text: text![Keyword::Store(Sentence::Start, 3)],
                ability_type: AbilityType::Activated(actions(1), TargetRequirement::None),
                delegates: vec![on_activated(|g, s, _| {
                    add_stored_mana(g, s.card_id(), 3);
                    Ok(())
                })],
            },
        ],
        config: CardConfig::default(),
    }
}

pub fn magical_resonator() -> CardDefinition {
    CardDefinition {
        name: CardName::MagicalResonator,
        cost: cost(1),
        image: rexard_images::artifact(RexardArtifactType::Bracers, "bracers_2"),
        card_type: CardType::Artifact,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::store_mana_on_play::<9>(),
            Ability {
                text: text![
                    Keyword::Take(Sentence::Start, 3),
                    ".",
                    "Use this ability only once per turn."
                ],
                ability_type: AbilityType::Activated(
                    Cost { mana: None, actions: 1, custom_cost: once_per_turn_cost() },
                    TargetRequirement::None,
                ),
                delegates: vec![on_activated(|g, _s, activated| {
                    mutations::take_stored_mana(g, activated.card_id(), 3, OnZeroStored::Sacrifice)
                        .map(|_| ())
                })],
            },
        ],
        config: CardConfig::default(),
    }
}

pub fn dark_grimoire() -> CardDefinition {
    CardDefinition {
        name: CardName::DarkGrimoire,
        cost: cost(3),
        image: rexard_images::get(RexardPack::MagicItems, "book_06_b"),
        card_type: CardType::Artifact,
        side: Side::Champion,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![simple_ability(
            text!["The first time each turn you take the 'draw card' action, draw another card."],
            Delegate::DrawCardAction(EventDelegate {
                requirement: face_up_in_play,
                mutation: |g, s, _| {
                    once_per_turn(g, s, &(), |g, s, _| {
                        mutations::draw_cards(g, s.side(), 1).map(|_| ())
                    })
                },
            }),
        )],
        config: CardConfig::default(),
    }
}
