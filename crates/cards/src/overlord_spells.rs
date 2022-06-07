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

//! Card definitions for the Spell card type & Overlord player

use data::card_definition::{CardConfig, CardDefinition, TargetRequirement};
use data::card_name::CardName;
use data::primitives::{CardType, Rarity, School, Side};
use rules::helpers::*;
use rules::text_macro::text;
use rules::{flags, mana, mutations};

pub fn gathering_dark() -> CardDefinition {
    CardDefinition {
        name: CardName::GatheringDark,
        cost: cost(5),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_45"),
        card_type: CardType::OverlordSpell,
        side: Side::Overlord,
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

pub fn overwhelming_power() -> CardDefinition {
    CardDefinition {
        name: CardName::OverwhelmingPower,
        cost: cost(10),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_46"),
        card_type: CardType::OverlordSpell,
        side: Side::Overlord,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![simple_ability(
            text!("Gain", mana_text(15)),
            on_cast(|g, s, _| {
                mana::gain(g, s.side(), 15);
                Ok(())
            }),
        )],
        config: CardConfig::default(),
    }
}

pub fn forced_march() -> CardDefinition {
    CardDefinition {
        name: CardName::ForcedMarch,
        cost: cost(1),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_47"),
        card_type: CardType::OverlordSpell,
        side: Side::Overlord,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![simple_ability(
            text!(
                "Place 2 level counters on each card in target room which didn't enter play this turn"
            ),
            on_cast(|g, _, played| {
                let targets = g
                    .defenders_and_occupants(played.target.room_id()?)
                    .filter(|card| {
                        flags::can_level_up_card(g, card.id)
                            && !flags::entered_play_this_turn(g, card.id)
                    })
                    .map(|card| card.id)
                    .collect::<Vec<_>>();
                for card_id in targets {
                    mutations::add_level_counters(g, card_id, 2)?;
                }

                Ok(())
            }))
        ],
        config: CardConfig {
            custom_targeting: Some(TargetRequirement::TargetRoom(|game, _, room_id| {
                game.defenders_and_occupants(room_id).any(|card| {
                    flags::can_level_up_card(game, card.id)
                        && !flags::entered_play_this_turn(game, card.id)
                })
            })),
            ..CardConfig::default()
        },
    }
}
