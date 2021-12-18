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

use model::card_definition::{AbilityText, CardDefinition, Keyword, NumericOperator, TextToken};
use model::card_state::CardState;
use model::delegates::Scope;
use model::game::GameState;
use model::primitives::{AbilityId, Side};
use protos::spelldawn::RulesText;

pub fn build(
    game: &GameState,
    card: &CardState,
    definition: &CardDefinition,
    _user_side: Side,
) -> RulesText {
    let mut result = vec![];
    for (index, ability) in definition.abilities.iter().enumerate() {
        result.push(match &ability.text {
            AbilityText::Text(text) => ability_text(text),
            AbilityText::TextFn(function) => {
                let tokens = function(game, Scope::new(AbilityId::new(card.id(), index)));
                ability_text(&tokens)
            }
        });
    }
    RulesText { text: result.join("\n") }
}

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
            TextToken::Keyword(keyword) => match keyword {
                Keyword::Play => "<b>\u{f0e7}Play:</b>".to_string(),
                Keyword::Dawn => "<b>\u{f0e7}Dawn:</b>".to_string(),
                Keyword::Dusk => "<b>\u{f0e7}Dusk:</b>".to_string(),
                Keyword::Score => "<b>\u{f0e7}Score:</b>".to_string(),
                Keyword::Combat => "<b>\u{f0e7}Combat:</b>".to_string(),
                Keyword::Strike(n) => format!("<b>Strike {}:</b>", n),
                Keyword::Store(n) => format!("<b>Store {}:</b>", n),
            },
            TextToken::Cost(cost) => format!("{}:", ability_text(cost)),
        })
    }
    result.join(" ")
}
