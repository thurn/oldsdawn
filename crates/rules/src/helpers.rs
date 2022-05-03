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
    AbilityType, AttackBoost, CardStats, Cost, SchemePoints, TriggerIndicator,
};
use data::delegates::{
    CardPlayed, Delegate, EventDelegate, MutationFn, QueryDelegate, RaidEnded, RequirementFn, Scope,
};
use data::game::GameState;
use data::game_actions::CardTarget;
use data::primitives::{
    AbilityId, ActionCount, AttackValue, BoostData, CardId, HealthValue, ManaValue, RaidId, Sprite,
    TurnNumber,
};
use data::text::{NumericOperator, TextToken};
use data::utils;

use crate::raid_actions;

pub fn number(number: impl Into<u32>) -> TextToken {
    TextToken::Number(NumericOperator::None, number.into())
}

pub fn add_number(number: impl Into<u32>) -> TextToken {
    TextToken::Number(NumericOperator::Add, number.into())
}

pub fn mana(value: ManaValue) -> TextToken {
    TextToken::Mana(value)
}

pub fn actions(value: ActionCount) -> TextToken {
    TextToken::Actions(value)
}

pub fn reminder(text: &'static str) -> TextToken {
    TextToken::Reminder(text.to_string())
}

pub fn cost_1_action() -> Cost {
    Cost { mana: None, actions: 1 }
}

/// Provides the cost for a card, with 1 action point required
pub fn cost(mana: ManaValue) -> Cost {
    Cost { mana: Some(mana), actions: 1 }
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

/// A RequirementFn which checks if the current `raid_id` matches the stored
/// [RaidId] for this `scope`.
pub fn matching_raid<T>(game: &GameState, scope: Scope, _: T) -> bool {
    utils::is_true(|| Some(game.ability_state(scope)?.raid_id? == game.data.raid.as_ref()?.raid_id))
}

/// Returns a standard [AbilityType] which does not notify the user when
/// triggered
pub fn silent() -> AbilityType {
    AbilityType::Standard(TriggerIndicator::Silent)
}

/// Returns a standard [AbilityType] which notifies the user when triggered
pub fn alert() -> AbilityType {
    AbilityType::Standard(TriggerIndicator::Alert)
}

/// A delegate which triggers when a card is cast
pub fn on_cast(mutation: MutationFn<CardPlayed>) -> Delegate {
    Delegate::CastCard(EventDelegate { requirement: this_card, mutation })
}

/// A delegate which triggers at dawn if a card is face up in play
pub fn at_dawn(mutation: MutationFn<TurnNumber>) -> Delegate {
    Delegate::Dawn(EventDelegate { requirement: face_up_in_play, mutation })
}

/// A delegate which triggers at dusk if a card is face up in play
pub fn at_dusk(mutation: MutationFn<TurnNumber>) -> Delegate {
    Delegate::Dusk(EventDelegate { requirement: face_up_in_play, mutation })
}

/// A minion combat delegate
pub fn combat(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::MinionCombatAbility(EventDelegate { requirement: this_card, mutation })
}

/// A delegate when a card is scored
pub fn on_overlord_score(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::OverlordScoreCard(EventDelegate { requirement: this_card, mutation })
}

/// A delegate which fires when a raid ends
pub fn on_raid_ended(
    requirement: RequirementFn<RaidEnded>,
    mutation: MutationFn<RaidEnded>,
) -> Delegate {
    Delegate::RaidEnd(EventDelegate { requirement, mutation })
}

pub fn add_vault_access<const N: u32>(requirement: RequirementFn<RaidId>) -> Delegate {
    Delegate::VaultAccessCount(QueryDelegate {
        requirement,
        transformation: |_, _, _, current| {
            println!("Adding {:?} to {:?}", N, current);
            current + N
        },
    })
}

pub fn add_sanctum_access<const N: u32>(requirement: RequirementFn<RaidId>) -> Delegate {
    Delegate::SanctumAccessCount(QueryDelegate {
        requirement,
        transformation: |_, _, _, current| current + N,
    })
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

/// Initiates a raid on the `target` room and stores the raid ID as ability
/// state.
pub fn initiate_raid(game: &mut GameState, scope: Scope, target: CardTarget) {
    initiate_raid_with_callback(game, scope, target, |_, _| {});
}

/// Initiates a raid on the `target` room and stores the raid ID as ability
/// state.
///
/// Invokes `on_begin` as soon as a [RaidId] is available.
pub fn initiate_raid_with_callback(
    game: &mut GameState,
    scope: Scope,
    target: CardTarget,
    on_begin: impl Fn(&mut GameState, RaidId),
) {
    raid_actions::initiate_raid(game, target.room_id().expect("Room Target"), |game, raid_id| {
        game.ability_state_mut(scope.ability_id()).raid_id = Some(raid_id);
        on_begin(game, raid_id);
    })
    .expect("Error initiating raid");
}
