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
    SchemePoints, TextToken,
};
use model::card_state::{CardPosition, CardPositionTypes, CardState};
use model::delegates;
use model::delegates::{CardMoved, Delegate, EventDelegate, MutationFn, QueryDelegate, Scope};
use model::game::GameState;
use model::primitives::{
    AbilityId, AttackValue, BoostData, CardId, HealthValue, ManaValue, Side, SpriteAddress,
    TurnNumber,
};
use rand::seq::IteratorRandom;
use std::cell::{RefCell, RefMut};
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
    game.card(scope.card_id()).position().in_play()
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

/// An ability which triggers at dawn if a card is in play
pub fn at_dawn(rules: CardText, mutation: MutationFn<TurnNumber>) -> Ability {
    Ability {
        text: rules,
        ability_type: AbilityType::Standard,
        delegates: vec![Delegate::OnDawn(EventDelegate { requirement: in_play, mutation })],
    }
}

/// An ability which triggers at dusk if a card is in play
pub fn at_dusk(rules: CardText, mutation: MutationFn<TurnNumber>) -> Ability {
    Ability {
        text: rules,
        ability_type: AbilityType::Standard,
        delegates: vec![Delegate::OnDusk(EventDelegate { requirement: in_play, mutation })],
    }
}

/// A minion combat ability
pub fn combat(rules: CardText, mutation: MutationFn<CardId>) -> Ability {
    Ability {
        text: rules,
        ability_type: AbilityType::Standard,
        delegates: vec![Delegate::OnMinionCombatAbility(EventDelegate {
            requirement: this_card,
            mutation,
        })],
    }
}

/// An ability when a card is scored
pub fn on_score(rules: CardText, mutation: MutationFn<CardId>) -> Ability {
    Ability {
        text: rules,
        ability_type: AbilityType::Standard,
        delegates: vec![Delegate::OnScoreScheme(EventDelegate {
            requirement: this_card,
            mutation,
        })],
    }
}

/// Give mana to the player who owns this delegate
pub fn gain_mana(game: &mut GameState, side: Side, amount: ManaValue) {
    game.player_mut(side).mana += amount;
}

/// Helper to create a [CardStats] with the given `base_attack` and [AttackBoost]
pub fn attack(base_attack: AttackValue, boost: AttackBoost) -> CardStats {
    CardStats { base_attack: Some(base_attack), attack_boost: Some(boost), ..CardStats::default() }
}

pub fn health(health: HealthValue) -> CardStats {
    CardStats { health: Some(health), ..CardStats::default() }
}

pub fn scheme_points(points: SchemePoints) -> CardStats {
    CardStats { scheme_points: Some(points), ..CardStats::default() }
}

pub fn move_card(game: &mut GameState, card_id: CardId, new_position: CardPosition) {
    let old_position = game.card(card_id).position();
    game.card_mut(card_id).move_to(new_position);

    dispatch::invoke_event(game, delegates::on_move_card, CardMoved { old_position, new_position });

    if old_position.in_deck() && new_position.in_hand() {
        dispatch::invoke_event(game, delegates::on_draw_card, card_id);
    }

    if old_position.in_hand() && new_position.in_play() {
        dispatch::invoke_event(game, delegates::on_play_card, card_id);
    }
}

/// Takes *up to* `amount` stored mana from a card and gives it to the player who owns this
/// delegate. Panics if there is no stored mana available.
pub fn take_stored_mana(game: &mut GameState, scope: Scope, amount: ManaValue) {
    let available = game.card(scope).data().stored_mana;
    assert!(available > 0, "No stored mana available!");
    let taken = std::cmp::min(available, amount);
    game.card_mut(scope).data_mut().stored_mana -= taken;
    dispatch::invoke_event(game, delegates::on_stored_mana_taken, scope.card_id());
    gain_mana(game, scope.side(), taken);
}

pub fn set_raid_ended(game: &mut GameState) {
    dispatch::invoke_event(game, delegates::on_raid_end, game.data().raid.expect("Active raid"));
    game.data_mut().raid = None;
}
