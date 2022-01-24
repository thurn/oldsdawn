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

use data::card_definition::CardDefinition;
use data::card_state::CardState;
use data::delegates::Scope;
use data::game::GameState;
use data::primitives::{AbilityId, CardSubtype, CardType, DamageType, Faction};
use data::text::{AbilityText, Keyword, KeywordKind, NumericOperator, TextToken};
use protos::spelldawn::{Node, RulesText};
use ui::card_info::SupplementalCardInfo;
use ui::core::Component;

/// Primary function which turns the current state of a card into its client
/// [RulesText] representation
pub fn build(game: &GameState, card: &CardState, definition: &CardDefinition) -> RulesText {
    let mut lines = vec![];
    for (index, ability) in definition.abilities.iter().enumerate() {
        lines.push(match &ability.text {
            AbilityText::Text(text) => ability_text(text),
            AbilityText::TextFn(function) => {
                let tokens = function(game, Scope::new(AbilityId::new(card.id, index)));
                ability_text(&tokens)
            }
        });
    }
    RulesText { text: lines.join("\n") }
}

/// Builds the supplemental info display for a card, which displays additional
/// help information and appears on long-press.
pub fn build_supplemental_info(
    game: &GameState,
    card: &CardState,
    definition: &CardDefinition,
) -> Node {
    let mut result = vec![card_type_line(definition)];
    let mut keywords = vec![];
    for (index, ability) in definition.abilities.iter().enumerate() {
        match &ability.text {
            AbilityText::Text(text) => find_keywords(text, &mut keywords),
            AbilityText::TextFn(function) => {
                let tokens = function(game, Scope::new(AbilityId::new(card.id, index)));
                find_keywords(&tokens, &mut keywords)
            }
        };
    }
    process_keywords(&mut keywords, &mut result);
    SupplementalCardInfo { info: result }.render()
}

/// Primary function for converting a sequence of [TextToken]s into a string
fn ability_text(tokens: &[TextToken]) -> String {
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
            TextToken::Mana(mana) => format!("{}\u{f06d}", mana),
            TextToken::Actions(mana) => format!("{}\u{f254}", mana),
            TextToken::Keyword(keyword) => match keyword {
                Keyword::Play => "<b>\u{f0e7}Play:</b>".to_string(),
                Keyword::Dawn => "<b>\u{f0e7}Dawn:</b>".to_string(),
                Keyword::Dusk => "<b>\u{f0e7}Dusk:</b>".to_string(),
                Keyword::Score => "<b>\u{f0e7}Score:</b>".to_string(),
                Keyword::Combat => "<b>\u{f0e7}Combat:</b>".to_string(),
                Keyword::Store(n) => format!("<b>Store {}:</b>", n),
                Keyword::DealDamage(amount, damage_type) => format!(
                    "Deal {} {} damage.",
                    amount,
                    match damage_type {
                        DamageType::Physical => "physical",
                        DamageType::Fire => "fire",
                        DamageType::Lightning => "lightning",
                        DamageType::Cold => "cold",
                    }
                ),
                Keyword::EndRaid => "End the raid.".to_string(),
            },
            TextToken::Cost(cost) => format!("{}: ", ability_text(cost)),
        })
    }

    result.join(" ")
}

fn card_type_line(definition: &CardDefinition) -> String {
    let mut result = String::new();
    result.push_str(match definition.card_type {
        CardType::Spell => "Spell",
        CardType::Weapon => "Weapon",
        CardType::Artifact => "Artifact",
        CardType::Sorcery => "Sorcery",
        CardType::Minion => "Minion",
        CardType::Project => "Project",
        CardType::Scheme => "Scheme",
        CardType::Upgrade => "Upgrade",
        CardType::Identity => "Identity",
    });

    if let Some(faction) = definition.config.faction {
        result.push_str(" • ");
        result.push_str(match faction {
            Faction::Prismatic => "Prismatic",
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
                output.push(
                    "\u{f0e7}<b>Play:</b> Triggers when this card enters the arena.".to_string(),
                );
            }
            KeywordKind::Dawn => {
                output.push(
                    "\u{f0e7}<b>Dawn:</b> Triggers at the start of the Champion's turn."
                        .to_string(),
                );
            }
            KeywordKind::Dusk => {
                output.push(
                    "\u{f0e7}<b>Dusk:</b> Triggers at the start of the Overlord's turn."
                        .to_string(),
                );
            }
            KeywordKind::Score => {
                output.push(
                    "\u{f0e7}<b>Score:</b> Triggers when the Overlord scores this card."
                        .to_string(),
                );
            }
            KeywordKind::Combat => {
                output.push(
                    "\u{f0e7}<b>Combat:</b> Triggers if this minion is not defeated in combat."
                        .to_string(),
                );
            }
            KeywordKind::Store => {
                output.push("<b>Store</b> Put \u{f06d} into this card.".to_string());
            }
            KeywordKind::DealDamage => {
                output.push(
                    "<b>Damage</b>: Causes the Champion to discard cards at random.".to_string(),
                );
            }
            _ => {}
        };
    }
}
