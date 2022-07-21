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

use card_helpers::{abilities, text, *};
use data::card_definition::{Ability, AbilityType, CardConfig, CardDefinition, TargetRequirement};
use data::card_name::CardName;
use data::primitives::{CardType, Rarity, School, Side};
use data::text::{Keyword, Sentence};
use rules::mutations;
use rules::mutations::OnZeroStored;

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
    }
}

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
            simple_ability(
                text![
                    Keyword::Dusk,
                    Keyword::Take(Sentence::Start, 3),
                    ".",
                    "When empty, draw a card."
                ],
                at_dusk(|g, s, _| {
                    mutations::take_stored_mana(g, s.card_id(), 3, OnZeroStored::Sacrifice)?;
                    if g.card(s.card_id()).data.stored_mana == 0 {
                        mutations::draw_cards(g, s.side(), 1)?;
                    }

                    // TODO: Consider not alerting on the first turn to avoid two popups
                    alert(g, s);
                    Ok(())
                }),
            ),
        ],
        config: CardConfig::default(),
    }
}

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
                        if mutations::unveil_project_for_free(g, s.card_id())? {
                            add_stored_mana(g, s.card_id(), 15);
                        }
                        mutations::take_stored_mana(g, s.card_id(), 3, OnZeroStored::Sacrifice)
                            .map(|_| ())
                    }),
                ],
            },
        ],
        config: CardConfig::default(),
    }
}

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
            simple_ability(
                text![
                    Keyword::Trap,
                    "If this card is in play, deal 2 damage plus 1 per level counter"
                ],
                on_accessed(|g, s, _| {
                    if g.card(s.card_id()).position().in_play() {
                        mutations::deal_damage(g, s, 2 + g.card(s.card_id()).data.card_level)?;
                        alert(g, s);
                    }

                    Ok(())
                }),
            ),
        ],
        config: CardConfig::default(),
    }
}
