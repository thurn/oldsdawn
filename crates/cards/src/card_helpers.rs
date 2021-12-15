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

//! Helpers for defining card behaviors. This file is intended be be used via wildcard import in
//! card definition files.

use crate::{dispatch, queries};
use model::card_definition::{
    Ability, AbilityType, AttackBoost, CardStats, CardText, Cost, Keyword, NumericOperator,
    TextToken,
};
use model::card_state::{CardPosition, CardState};
use model::delegates;
use model::delegates::{Delegate, EventDelegate, MutationFn, QueryDelegate, Scope};
use model::game::GameState;
use model::primitives::{AbilityId, AttackValue, BoostData, CardId, ManaValue, SpriteAddress};
use std::sync::Arc;

/// Provides the rules text for a card
pub fn text(text: impl Into<String>) -> TextToken {
    TextToken::Literal(text.into())
}

pub fn number(number: impl Into<u32>) -> TextToken {
    TextToken::Number(NumericOperator::None, number.into())
}

pub fn add_number(number: impl Into<u32>) -> TextToken {
    TextToken::Number(NumericOperator::Add, number.into())
}

pub fn mana_symbol(value: ManaValue) -> TextToken {
    TextToken::Mana(value)
}

pub fn mana_cost_text(value: ManaValue) -> TextToken {
    TextToken::Cost(vec![mana_symbol(value)])
}

pub fn keyword(keyword: Keyword) -> TextToken {
    TextToken::Keyword(keyword)
}

/// Provides the cost for a card
pub fn cost(mana: ManaValue) -> Cost {
    Cost { mana, actions: 1 }
}

/// Provides an image for a card
pub fn sprite(text: &str) -> SpriteAddress {
    SpriteAddress(text.to_owned())
}

/// RequirementFn which always returns true
pub fn always<T>(_: &GameState, _: Scope, _: T) -> bool {
    true
}

/// RequirementFn that this delegate's card is currently in play
pub fn in_play<T>(game: &GameState, scope: Scope, _: T) -> bool {
    game.card(scope.card_id()).position.in_play()
}

/// A RequirementFn which restricts delegates to only listen to events for their own card.
pub fn this_card(game: &GameState, scope: Scope, card_id: impl Into<CardId>) -> bool {
    scope.card_id() == card_id.into()
}

/// A RequirementFn which restricts delegates to only listen to events for their own ability.
pub fn this_ability(game: &GameState, scope: Scope, ability_id: impl Into<AbilityId>) -> bool {
    scope.ability_id() == ability_id.into()
}

/// An ability which triggers when a card is played
pub fn on_play(rules: CardText, mutation: MutationFn<CardId>) -> Ability {
    Ability {
        text: rules,
        ability_type: AbilityType::Standard,
        delegates: vec![Delegate::OnPlayCard(EventDelegate { requirement: this_card, mutation })],
    }
}

/// Give mana to the player who owns this delegate
pub fn gain_mana(game: &mut GameState, scope: Scope, amount: ManaValue) {
    game.player_state_mut(scope.side()).mana += amount;
}

/// Helper to create a [CardStats] with the given `base_attack` and [AttackBoost]
pub fn attack(base_attack: AttackValue, boost: AttackBoost) -> CardStats {
    CardStats { base_attack: Some(base_attack), attack_boost: Some(boost), ..CardStats::default() }
}

/// Overwrites the value of [CardState::boost_count] to match the provided [BoostData]
pub fn write_boost(game: &mut GameState, scope: Scope, data: BoostData) {
    game.card_mut(data).data.boost_count = data.count
}

/// Applies this card's `attack_boost` stat a number of times equal to its [CardState::boost_count]
pub fn add_boost(
    game: &GameState,
    scope: Scope,
    card_id: CardId,
    current: AttackValue,
) -> AttackValue {
    let boost_count = queries::boost_count(game, card_id);
    let bonus = queries::stats(game, card_id).attack_boost.expect("Expected boost").bonus;

    current + (boost_count * bonus)
}

/// Set the boost count to zero for the card in `scope`
pub fn clear_boost<T>(game: &mut GameState, scope: Scope, _: T) {
    game.card_mut(scope).data.boost_count = 0
}

/// The standard weapon ability; applies an attack boost for the duration of a single encounter.
pub fn encounter_boost() -> Ability {
    Ability {
        text: CardText::TextFn(|g, s| {
            let boost = queries::stats(g, s).attack_boost.expect("attack_boost");
            vec![mana_cost_text(boost.cost), add_number(boost.bonus), text("Attack")]
        }),
        ability_type: AbilityType::Encounter,
        delegates: vec![
            Delegate::OnActivateBoost(EventDelegate::new(this_card, write_boost)),
            Delegate::GetAttackValue(QueryDelegate::new(this_card, add_boost)),
            Delegate::OnEncounterEnd(EventDelegate::new(always, clear_boost)),
        ],
    }
}

pub fn store_mana() -> Ability {
    Ability {
        text: CardText::TextFn(|g, s| {
            let store_amount = queries::stats(g, s).store_mana.expect("store_mana");
            vec![keyword(Keyword::Play), keyword(Keyword::Store), mana_symbol(store_amount)]
        }),
        ability_type: AbilityType::Standard,
        delegates: vec![],
    }
}
