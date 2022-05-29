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

//! Card definitions for the Project card type

use data::card_definition::{Ability, AbilityType, CardConfig, CardDefinition, TargetRequirement};
use data::card_name::CardName;
use data::primitives::{CardType, DamageType, Rarity, School, Side};
use data::text::{Keyword, Sentence};
use linkme::distributed_slice;
use rules::helpers::*;
use rules::mutations::{take_stored_mana, OnZeroStored};
use rules::{abilities, mutations, text, DEFINITIONS};

pub fn initialize() {}

#[distributed_slice(DEFINITIONS)]
pub fn gold_mine() -> CardDefinition {
    CardDefinition {
        name: CardName::GoldMine,
        cost: cost(4),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_32"),
        card_type: CardType::Project,
        side: Side::Overlord,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![
            Ability {
                text: text![Keyword::Unveil, "at Dusk, then", Keyword::Store(Sentence::Start, 12)],
                ability_type: AbilityType::Standard,
                delegates: vec![unveil_at_dusk(), store_mana_on_unveil::<12>()],
            },
            Ability {
                text: text![Keyword::Dusk, Keyword::Take(Sentence::Start, 3)],
                ability_type: AbilityType::Standard,
                delegates: vec![at_dusk(|g, s, _| {
                    mutations::take_stored_mana(g, s.card_id(), 3, OnZeroStored::Sacrifice);
                    alert(g, s);
                })],
            },
        ],
        config: CardConfig::default(),
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn gemcarver() -> CardDefinition {
    CardDefinition {
        name: CardName::Gemcarver,
        cost: cost(2),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_33"),
        card_type: CardType::Project,
        side: Side::Overlord,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![
            Ability {
                text: text![Keyword::Unveil, "at Dusk, then", Keyword::Store(Sentence::Start, 9)],
                ability_type: AbilityType::Standard,
                delegates: vec![unveil_at_dusk(), store_mana_on_unveil::<9>()],
            },
            Ability {
                text: text![
                    Keyword::Dusk,
                    Keyword::Take(Sentence::Start, 3),
                    ".",
                    "When empty, draw a card."
                ],
                ability_type: AbilityType::Standard,
                delegates: vec![at_dusk(|g, s, _| {
                    mutations::take_stored_mana(g, s.card_id(), 3, OnZeroStored::Sacrifice);
                    if g.card(s.card_id()).data.stored_mana == 0 {
                        mutations::draw_cards(g, s.side(), 1);
                    }
                    alert(g, s);
                })],
            },
        ],
        config: CardConfig::default(),
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn coinery() -> CardDefinition {
    CardDefinition {
        name: CardName::Coinery,
        cost: cost(2),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_33"),
        card_type: CardType::Project,
        side: Side::Overlord,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![
            text_only_ability(text![
                Keyword::Unveil,
                "when activated, then",
                Keyword::Store(Sentence::Start, 15)
            ]),
            Ability {
                text: text![Keyword::Take(Sentence::Start, 3)],
                ability_type: AbilityType::Activated(actions(1), TargetRequirement::None),
                delegates: vec![
                    activate_while_face_down(),
                    face_down_ability_cost(),
                    on_activated(|g, s, _| {
                        if mutations::unveil_project_for_free(g, s.card_id()) {
                            add_stored_mana(g, s.card_id(), 15);
                        }
                        take_stored_mana(g, s.card_id(), 3, OnZeroStored::Sacrifice);
                    }),
                ],
            },
        ],
        config: CardConfig::default(),
    }
}

#[distributed_slice(DEFINITIONS)]
pub fn pit_trap() -> CardDefinition {
    CardDefinition {
        name: CardName::PitTrap,
        cost: cost(2),
        image: sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_34"),
        card_type: CardType::Project,
        side: Side::Overlord,
        school: School::Time,
        rarity: Rarity::Common,
        abilities: vec![
            abilities::level_up(),
            Ability {
                text: text![
                    Keyword::Trap,
                    "If this card is in play, deal 2 damage plus 1 per level counter"
                ],
                ability_type: AbilityType::Standard,
                delegates: vec![on_accessed(|g, s, _| {
                    if g.card(s.card_id()).position().in_play() {
                        mutations::deal_damage(
                            g,
                            s,
                            DamageType::Physical,
                            2 + g.card(s.card_id()).data.card_level,
                        );
                        alert(g, s);
                    }
                })],
            },
        ],
        config: CardConfig::default(),
    }
}
