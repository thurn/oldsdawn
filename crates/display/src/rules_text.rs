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

//! Tools for rendering the text on a card

use core_ui::{icons, rendering};
use data::card_definition::{Ability, AbilityType, CardDefinition, Cost};
use data::card_state::CardState;
use data::delegates::Scope;
use data::game::GameState;
use data::primitives::{AbilityId, AbilityIndex, CardSubtype, CardType, Faction};
use data::text::{
    AbilityText, DamageWord, Keyword, KeywordKind, NumericOperator, Sentence, TextToken,
};
use game_ui::card_info::SupplementalCardInfo;
use protos::spelldawn::{Node, RulesText};

/// Primary function which turns the current state of a card into its client
/// [RulesText] representation
pub fn build(game: &GameState, card: &CardState, definition: &CardDefinition) -> RulesText {
    let mut lines = vec![];
    for (index, ability) in definition.abilities.iter().enumerate() {
        let mut line = String::new();
        if let AbilityType::Activated(cost, _) = &ability.ability_type {
            line.push_str(&ability_cost_string(cost));
        }

        line.push_str(&ability_text(game, AbilityId::new(card.id, index), ability));

        lines.push(line);
    }

    if let Some(breach) = definition.config.stats.breach {
        lines.push(process_text_tokens(&[TextToken::Keyword(Keyword::Breach(breach))]));
    }

    RulesText { text: lines.join("\n") }
}

/// Builds the rules text for a single [Ability], not including its cost (if
/// any).
pub fn ability_text(game: &GameState, ability_id: AbilityId, ability: &Ability) -> String {
    match &ability.text {
        AbilityText::Text(text) => process_text_tokens(text),
        AbilityText::TextFn(function) => {
            let tokens = function(game, Scope::new(ability_id));
            process_text_tokens(&tokens)
        }
    }
}

/// Builds the supplemental info display for a card, which displays additional
/// help information and appears on long-press.
///
/// If an `ability_index` is provided, only supplemental info for that index is
/// returned. Otherwise, supplemental info for all abilities is returned.
pub fn build_supplemental_info(
    game: &GameState,
    card: &CardState,
    ability_index: Option<AbilityIndex>,
) -> Option<Node> {
    let definition = rules::get(card.name);
    let mut result = vec![card_type_line(definition)];
    let mut keywords = vec![];
    for (index, ability) in definition.abilities.iter().enumerate() {
        if matches!(ability_index, Some(i) if i.value() != index) {
            continue;
        }

        match &ability.text {
            AbilityText::Text(text) => find_keywords(text, &mut keywords),
            AbilityText::TextFn(function) => {
                let tokens = function(game, Scope::new(AbilityId::new(card.id, index)));
                find_keywords(&tokens, &mut keywords)
            }
        };
    }

    if definition.config.stats.breach.is_some() {
        keywords.push(KeywordKind::Breach);
    }

    process_keywords(&mut keywords, &mut result);
    rendering::component(SupplementalCardInfo::new(result))
}

fn ability_cost_string(cost: &Cost<AbilityId>) -> String {
    let mut actions = icons::ACTION.repeat(cost.actions as usize);

    if let Some(mana) = cost.mana {
        if mana > 0 {
            actions.push_str(&format!(",{}{}", mana, icons::MANA));
        }
    }

    actions.push_str(&format!(" {} ", icons::ARROW));
    actions
}

/// Primary function for converting a sequence of [TextToken]s into a string
fn process_text_tokens(tokens: &[TextToken]) -> String {
    let mut result = vec![];
    for token in tokens {
        result.push(match token {
            TextToken::Literal(text) => text.clone(),
            TextToken::Number(operator, number) => format!(
                "{}{}",
                match operator {
                    NumericOperator::None => "",
                    NumericOperator::Add => "+",
                },
                number
            ),
            TextToken::Mana(mana) => format!("{}{}", mana, icons::MANA),
            TextToken::Actions(actions) => format!("{}{}", actions, icons::ACTION),
            TextToken::Keyword(keyword) => match keyword {
                Keyword::Play => "<b>\u{f0e7}Play:</b>".to_string(),
                Keyword::Dawn => "<b>\u{f0e7}Dawn:</b>".to_string(),
                Keyword::Dusk => "<b>\u{f0e7}Dusk:</b>".to_string(),
                Keyword::Score => format!("<b>{}Score:</b>", icons::TRIGGER),
                Keyword::Combat => format!("<b>{}Combat:</b>", icons::TRIGGER),
                Keyword::Encounter => format!("<b>{}Encounter:</b>", icons::TRIGGER),
                Keyword::Unveil => "<b>Unveil</b>".to_string(),
                Keyword::SuccessfulRaid => format!("<b>{}Successful Raid:</b>", icons::TRIGGER),
                Keyword::Store(sentence_position, n) => {
                    format!(
                        "<b>{}</b>{}{}{}",
                        match sentence_position {
                            Sentence::Start => "Store",
                            Sentence::Internal => "store",
                        },
                        icons::NON_BREAKING_SPACE,
                        n,
                        icons::MANA
                    )
                }
                Keyword::Take(sentence_position, n) => format!(
                    "{}{}{}{}",
                    match sentence_position {
                        Sentence::Start => "Take",
                        Sentence::Internal => "take",
                    },
                    icons::NON_BREAKING_SPACE,
                    n,
                    icons::MANA
                ),
                Keyword::DealDamage(word, amount, damage_type) => format!(
                    "{} {} {}",
                    match word {
                        DamageWord::DealStart => "Deal",
                        DamageWord::DealInternal => "deal",
                        DamageWord::TakeStart => "Take",
                        DamageWord::TakeInternal => "take",
                    },
                    amount,
                    damage_type
                ),
                Keyword::InnerRoom(sentence_position) => match sentence_position {
                    Sentence::Start => "Inner room",
                    Sentence::Internal => "inner room",
                }
                .to_string(),
                Keyword::Breach(breach) => {
                    format!("<b>Breach</b>{}{}", icons::NON_BREAKING_SPACE, breach)
                }
                Keyword::LevelUp => "<b>Level Up</b>".to_string(),
                Keyword::Trap => format!("<b>{}Trap:</b>", icons::TRIGGER),
                Keyword::Construct => "<b>Construct</b>".to_string(),
            },
            TextToken::Reminder(text) => format!("<i>{}</i>", text),
            TextToken::Cost(cost) => format!("{}: ", process_text_tokens(cost)),
        })
    }

    let string = result.join(" ");
    string.replace(" .", ".").replace(" ,", ",") // Don't have punctuation exist
                                                 // on its own
}

fn card_type_line(definition: &CardDefinition) -> String {
    let mut result = String::new();
    result.push_str(match definition.card_type {
        CardType::ChampionSpell => "Spell",
        CardType::Weapon => "Weapon",
        CardType::Artifact => "Artifact",
        CardType::OverlordSpell => "Spell",
        CardType::Minion => "Minion",
        CardType::Project => "Project",
        CardType::Scheme => "Scheme",
        CardType::Identity => "Identity",
    });

    if let Some(faction) = definition.config.faction {
        result.push_str(" • ");
        result.push_str(match faction {
            Faction::Prismatic => "Prismatic",
            Faction::Construct => "Construct",
            Faction::Mortal => "Mortal",
            Faction::Abyssal => "Abyssal",
            Faction::Infernal => "Infernal",
        });
    }

    for subtype in &definition.config.subtypes {
        result.push_str(" • ");
        result.push_str(match subtype {
            CardSubtype::Silvered => "Silvered",
        });
    }

    result
}

fn find_keywords(tokens: &[TextToken], keywords: &mut Vec<KeywordKind>) {
    keywords.extend(tokens.iter().filter_map(|t| {
        if let TextToken::Keyword(k) = t {
            Some(k.kind())
        } else {
            None
        }
    }));
}

fn process_keywords(keywords: &mut Vec<KeywordKind>, output: &mut Vec<String>) {
    keywords.sort();
    keywords.dedup();

    for keyword in keywords {
        match keyword {
            KeywordKind::Play => {
                output.push("<b>Play:</b> Triggers when this card enters the arena.".to_string());
            }
            KeywordKind::Dawn => {
                output
                    .push("<b>Dawn:</b> Triggers at the start of the Champion's turn.".to_string());
            }
            KeywordKind::Dusk => {
                output
                    .push("<b>Dusk:</b> Triggers at the start of the Overlord's turn.".to_string());
            }
            KeywordKind::Score => {
                output
                    .push("<b>Score:</b> Triggers when the Overlord scores this card.".to_string());
            }
            KeywordKind::Combat => {
                output.push(
                    "<b>Combat:</b> Triggers if this minion is not defeated during a raid."
                        .to_string(),
                );
            }
            KeywordKind::Encounter => {
                output.push(
                    "<b>Encounter:</b> Triggers when this minion is approached during a raid."
                        .to_string(),
                );
            }
            KeywordKind::Unveil => {
                output.push("<b>Unveil:</b> Pay cost and turn face up (if able)".to_string());
            }
            KeywordKind::SuccessfulRaid => {
                output.push(
                    "<b>Successful Raid:</b> Triggers after the access phase of a raid."
                        .to_string(),
                );
            }
            KeywordKind::Store => {
                output.push(format!(
                    "<b>Store:</b> Place {} on this card to take later.",
                    icons::MANA
                ));
            }
            KeywordKind::DealDamage => {
                output.push(
                    "<b>Damage:</b> Causes the Champion to discard cards at random.".to_string(),
                );
            }
            KeywordKind::InnerRoom => {
                output.push("<b>Inner Room:</b> The Sanctum, Vault or Crypts.".to_string())
            }
            KeywordKind::Breach => output.push(
                "<b>Breach:</b> Allows this weapon to bypass some amount of Shield.".to_string(),
            ),
            KeywordKind::LevelUp => output.push(
                "<b>Level Up:</b> This card gets level counters when its room is leveled up."
                    .to_string(),
            ),
            KeywordKind::Trap => output.push(
                "<b>Trap:</b> Triggers when this card is accessed during a raid.".to_string(),
            ),
            KeywordKind::Construct => output.push(
                "<b>Construct:</b> Goes to discard pile when defeated. Damage with any weapon."
                    .to_string(),
            ),
            _ => {}
        };
    }
}
