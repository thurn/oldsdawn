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
use data::game_actions::{CardPromptAction, GamePrompt};
use data::primitives::{CardType, ColdDamage, Faction, Rarity, School, Side};
use data::text::Keyword;
use linkme::distributed_slice;
use rules::helpers::*;
use rules::mana::ManaPurpose;
use rules::mutations::SetPrompt;
use rules::{abilities, mana, mutations, text, DEFINITIONS};

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
                    let mut responses = vec![CardPromptAction::EndRaid];
                    if mana::get(g, Side::Champion, ManaPurpose::PayForPrompt) >= 5 {
                        responses.push(CardPromptAction::LoseMana(Side::Champion, 5))
                    }
                    if g.champion.actions >= 2 {
                        responses.push(CardPromptAction::LoseActions(Side::Champion, 2))
                    }
                    mutations::set_prompt(
                        g,
                        Side::Champion,
                        SetPrompt::CardPrompt,
                        GamePrompt::card_actions(responses),
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
