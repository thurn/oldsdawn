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

use data::card_definition::{AttackBoost, CardStats, Cost, CustomCost, SchemePoints};
use data::delegates::{
    AbilityActivated, CardPlayed, Delegate, EventDelegate, MutationFn, QueryDelegate, RaidEnded,
    RaidStart, RequirementFn, Scope,
};
use data::game::GameState;
use data::game_actions::CardTarget;
use data::primitives::{
    AbilityId, ActionCount, AttackValue, BoostData, CardId, HealthValue, ManaValue, RaidId, RoomId,
    Sprite, TurnNumber,
};
use data::text::{NumericOperator, TextToken};
use data::updates::GameUpdate;
use data::utils;

use crate::raid_actions;

pub fn number(number: impl Into<u32>) -> TextToken {
    TextToken::Number(NumericOperator::None, number.into())
}

pub fn add_number(number: impl Into<u32>) -> TextToken {
    TextToken::Number(NumericOperator::Add, number.into())
}

pub fn mana_text(value: ManaValue) -> TextToken {
    TextToken::Mana(value)
}

pub fn actions_text(value: ActionCount) -> TextToken {
    TextToken::Actions(value)
}

pub fn reminder(text: &'static str) -> TextToken {
    TextToken::Reminder(text.to_string())
}

/// A [Cost] which requires no mana and `actions` action points.
pub fn actions(actions: ActionCount) -> Cost<AbilityId> {
    Cost { mana: None, actions, custom_cost: None }
}

/// Provides the cost for a card, with 1 action point required and `mana` mana
/// points
pub fn cost(mana: ManaValue) -> Cost<CardId> {
    Cost { mana: Some(mana), actions: 1, custom_cost: None }
}

/// [Cost] for an identity card
pub fn identity_cost() -> Cost<CardId> {
    Cost::default()
}

/// [Cost] for a scheme card
pub fn scheme_cost() -> Cost<CardId> {
    Cost { mana: None, actions: 1, custom_cost: None }
}

/// A [CustomCost] which allows an ability to be activated once per turn.
///
/// Stores turn data in ability state. Never returns `None`.
pub fn once_per_turn_ability() -> Option<CustomCost<AbilityId>> {
    Some(CustomCost {
        can_pay: |game, ability_id| {
            utils::is_false(|| Some(game.ability_state(ability_id)?.turn? == game.data.turn))
        },
        pay: |game, ability_id| {
            game.ability_state_mut(ability_id).turn = Some(game.data.turn);
        },
    })
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
    utils::is_true(|| {
        Some(game.ability_state(scope.ability_id())?.raid_id? == game.data.raid.as_ref()?.raid_id)
    })
}

/// Predicate checking if a room is an inner room
pub fn is_inner_room(room_id: RoomId) -> bool {
    room_id == RoomId::Vault || room_id == RoomId::Sanctum || room_id == RoomId::Crypts
}

/// Pushes a [GameUpdate] indicating the ability represented by [Scope] should
/// have a trigger animation shown in the UI.
pub fn alert(game: &mut GameState, scope: &Scope) {
    game.updates.push(GameUpdate::AbilityTriggered(scope.ability_id()));
}

/// Invokes [alert] if the provided `number` is not zero.
pub fn alert_if_nonzero(game: &mut GameState, scope: &Scope, number: u32) {
    if number > 0 {
        alert(game, scope);
    }
}

/// A delegate which triggers when a card is cast
pub fn on_cast(mutation: MutationFn<CardPlayed>) -> Delegate {
    Delegate::CastCard(EventDelegate { requirement: this_card, mutation })
}

/// A [Delegate] which triggers when an ability is activated
pub fn on_activated(mutation: MutationFn<AbilityActivated>) -> Delegate {
    Delegate::ActivateAbility(EventDelegate { requirement: this_ability, mutation })
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

/// Delegate which fires when a raid starts
pub fn on_raid_start(
    requirement: RequirementFn<RaidStart>,
    mutation: MutationFn<RaidStart>,
) -> Delegate {
    Delegate::RaidStart(EventDelegate { requirement, mutation })
}

/// Delegate which fires when the 'access' phase of a raid begins.
pub fn on_raid_access_start(
    requirement: RequirementFn<RaidId>,
    mutation: MutationFn<RaidId>,
) -> Delegate {
    Delegate::RaidAccessStart(EventDelegate { requirement, mutation })
}

/// A delegate which fires when a raid ends
pub fn on_raid_ended(
    requirement: RequirementFn<RaidEnded>,
    mutation: MutationFn<RaidEnded>,
) -> Delegate {
    Delegate::RaidEnd(EventDelegate { requirement, mutation })
}

/// A delegate which fires when a raid ends in success
pub fn on_raid_success(
    requirement: RequirementFn<RaidId>,
    mutation: MutationFn<RaidId>,
) -> Delegate {
    Delegate::RaidSuccess(EventDelegate { requirement, mutation })
}

/// A delegate which fires when a raid ends in failure
pub fn on_raid_failure(
    requirement: RequirementFn<RaidId>,
    mutation: MutationFn<RaidId>,
) -> Delegate {
    Delegate::RaidFailure(EventDelegate { requirement, mutation })
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

/// Invokes `function` at most once per turn.
///
/// Stores ability state to track the last-invoked turn number
pub fn once_per_turn<T>(game: &mut GameState, scope: Scope, data: T, function: MutationFn<T>) {
    if utils::is_false(|| Some(game.ability_state(scope.ability_id())?.turn? == game.data.turn)) {
        save_turn(game, scope);
        function(game, scope, data)
    }
}

/// Stores the current turn as ability state for the provided `ability_id`.
pub fn save_turn(game: &mut GameState, ability_id: impl Into<AbilityId>) {
    game.ability_state_mut(ability_id.into()).turn = Some(game.data.turn);
}

/// Helper to store the provided [RaidId] as ability state for this [Scope].
pub fn save_raid_id(game: &mut GameState, ability_id: impl Into<AbilityId>, raid_id: RaidId) {
    game.ability_state_mut(ability_id.into()).raid_id = Some(raid_id);
}

/// Add `amount` to the stored mana in a card. Returns the new stored amount.
pub fn add_stored_mana(game: &mut GameState, card_id: CardId, amount: ManaValue) -> ManaValue {
    game.card_mut(card_id).data.stored_mana += amount;
    game.card(card_id).data.stored_mana
}
