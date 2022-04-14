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

//! Helpers for defining card behaviors. This file is intended be be used via
//! wildcard import in card definition files.

use data::card_definition::{
    Ability, AbilityType, AttackBoost, CardStats, Cost, SchemePoints, TriggerIndicator,
};
use data::delegates::{Delegate, EventDelegate, MutationFn, Scope};
use data::game::GameState;
use data::primitives::{
    AbilityId, AttackValue, BoostData, CardId, HealthValue, ManaValue, Sprite, TurnNumber,
};
use data::text::{AbilityText, NumericOperator, TextToken};

pub fn number(number: impl Into<u32>) -> TextToken {
    TextToken::Number(NumericOperator::None, number.into())
}

pub fn add_number(number: impl Into<u32>) -> TextToken {
    TextToken::Number(NumericOperator::Add, number.into())
}

pub fn mana(value: ManaValue) -> TextToken {
    TextToken::Mana(value)
}

pub fn scheme_cost() -> Cost {
    Cost { mana: None, actions: 1 }
}

/// Provides the cost for a card, with 1 action point required
pub fn cost(mana: ManaValue) -> Cost {
    Cost { mana: if mana == 0 { None } else { Some(mana) }, actions: 1 }
}

/// Provides an image for a card
pub fn sprite(text: &str) -> Sprite {
    Sprite::new(text.to_string())
}

/// RequirementFn which always returns true
pub fn always<T>(_: &GameState, _: Scope, _: T) -> bool {
    true
}

/// RequirementFn that this delegate's card is currently face up & in play
pub fn face_up_in_play<T>(game: &GameState, scope: Scope, _: T) -> bool {
    let card = game.card(scope.card_id());
    card.is_face_up() && card.position().in_play()
}

/// RequirementFn that this delegate's card is currently face down & in play
pub fn face_down_in_play<T>(game: &GameState, scope: Scope, _: T) -> bool {
    let card = game.card(scope.card_id());
    card.is_face_down() && card.position().in_play()
}

/// A RequirementFn which restricts delegates to only listen to events for their
/// own card.
pub fn this_card(_game: &GameState, scope: Scope, card_id: impl Into<CardId>) -> bool {
    scope.card_id() == card_id.into()
}

/// A RequirementFn which restricts delegates to only listen to events for their
/// own ability.
pub fn this_ability(_game: &GameState, scope: Scope, ability_id: impl Into<AbilityId>) -> bool {
    scope.ability_id() == ability_id.into()
}

/// A RequirementFn which restricts delegates to only listen to [BoostData]
/// events matching their card.
pub fn this_boost(_game: &GameState, scope: Scope, boost_data: BoostData) -> bool {
    scope.card_id() == boost_data.card_id
}

/// An ability which triggers when a card is cast
pub fn on_cast(rules: AbilityText, mutation: MutationFn<CardId>) -> Ability {
    Ability {
        text: rules,
        ability_type: AbilityType::Standard(TriggerIndicator::Silent),
        delegates: vec![Delegate::CastCard(EventDelegate { requirement: this_card, mutation })],
    }
}

/// An ability which triggers when a card is played
pub fn on_play(rules: AbilityText, mutation: MutationFn<CardId>) -> Ability {
    Ability {
        text: rules,
        ability_type: AbilityType::Standard(TriggerIndicator::Silent),
        delegates: vec![Delegate::PlayCard(EventDelegate { requirement: this_card, mutation })],
    }
}

/// An ability which triggers at dawn if a card is face up in play
pub fn at_dawn(rules: AbilityText, mutation: MutationFn<TurnNumber>) -> Ability {
    Ability {
        text: rules,
        ability_type: AbilityType::Standard(TriggerIndicator::Alert),
        delegates: vec![Delegate::Dawn(EventDelegate { requirement: face_up_in_play, mutation })],
    }
}

/// An ability which triggers at dusk if a card is face up in play
pub fn at_dusk(rules: AbilityText, mutation: MutationFn<TurnNumber>) -> Ability {
    Ability {
        text: rules,
        ability_type: AbilityType::Standard(TriggerIndicator::Alert),
        delegates: vec![Delegate::Dusk(EventDelegate { requirement: face_up_in_play, mutation })],
    }
}

/// A minion combat ability
pub fn combat(rules: AbilityText, mutation: MutationFn<CardId>) -> Ability {
    Ability {
        text: rules,
        ability_type: AbilityType::Standard(TriggerIndicator::Silent),
        delegates: vec![Delegate::MinionCombatAbility(EventDelegate {
            requirement: this_card,
            mutation,
        })],
    }
}

/// An ability when a card is scored
pub fn on_overlord_score(rules: AbilityText, mutation: MutationFn<CardId>) -> Ability {
    Ability {
        text: rules,
        ability_type: AbilityType::Standard(TriggerIndicator::Silent),
        delegates: vec![Delegate::OverlordScoreCard(EventDelegate {
            requirement: this_card,
            mutation,
        })],
    }
}

/// Helper to create a [CardStats] with the given base [AttackValue]
pub fn base_attack(base_attack: AttackValue) -> CardStats {
    CardStats { base_attack: Some(base_attack), ..CardStats::default() }
}

/// Helper to create a [CardStats] with the given base [AttackValue] and
/// [AttackBoost]
pub fn attack(base_attack: AttackValue, boost: AttackBoost) -> CardStats {
    CardStats { base_attack: Some(base_attack), attack_boost: Some(boost), ..CardStats::default() }
}

/// Helper to create a [CardStats] with the given [HealthValue]
pub fn health(health: HealthValue) -> CardStats {
    CardStats { health: Some(health), ..CardStats::default() }
}

/// Helper to create a [CardStats] with the given [SchemePoints] and mark it as
/// a card which can be leveled up.
pub fn scheme_points(points: SchemePoints) -> CardStats {
    CardStats { scheme_points: Some(points), can_level_up: true, ..CardStats::default() }
}
