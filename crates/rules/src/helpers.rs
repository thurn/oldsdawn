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

use anyhow::Result;
use data::card_definition::{
    Ability, AbilityType, AttackBoost, CardStats, Cost, CustomCost, SchemePoints, SpecialEffects,
};
use data::card_state::CardPosition;
use data::delegates::{
    AbilityActivated, CardPlayed, Delegate, EventDelegate, MutationFn, QueryDelegate, RaidEnded,
    RaidStart, RequirementFn, Scope, TransformationFn, UsedWeapon,
};
use data::game::GameState;
use data::game_actions::{CardPromptAction, CardTarget, GamePrompt};
use data::primitives::{
    AbilityId, ActionCount, AttackValue, CardId, DamageType, HasAbilityId, HasCardId, HealthValue,
    ManaValue, RaidId, RoomId, Side, Sprite, TurnNumber,
};
use data::special_effects::Projectile;
use data::text::{AbilityText, NumericOperator, TextToken};
use data::updates::{GameUpdate, InitiatedBy};
use data::utils;

use crate::mana::ManaPurpose;
use crate::mutations::SetPrompt;
use crate::{mana, mutations, queries, raid};

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

/// An ability which only exists to add text to a card.
pub fn text_only_ability(text: AbilityText) -> Ability {
    Ability { text, ability_type: AbilityType::TextOnly, delegates: vec![] }
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
pub fn once_per_turn_cost() -> Option<CustomCost<AbilityId>> {
    Some(CustomCost {
        can_pay: |game, ability_id| {
            utils::is_false(|| Some(game.ability_state(ability_id)?.turn? == game.data.turn))
        },
        pay: |game, ability_id| {
            game.ability_state_mut(ability_id).turn = Some(game.data.turn);
            Ok(())
        },
    })
}

/// Provides an image for a card
pub fn sprite(text: &str) -> Sprite {
    Sprite::new(text.to_string())
}

/// Creates a standard [Ability] with a single [Delegate].
pub fn simple_ability(text: AbilityText, delegate: Delegate) -> Ability {
    Ability { text, ability_type: AbilityType::Standard, delegates: vec![delegate] }
}

/// RequirementFn which always returns true
pub fn always<T>(_: &GameState, _: Scope, _: &T) -> bool {
    true
}

/// RequirementFn that this delegate's card is currently face up & in play
pub fn face_up_in_play<T>(game: &GameState, scope: Scope, _: &T) -> bool {
    let card = game.card(scope.card_id());
    card.is_face_up() && card.position().in_play()
}

/// RequirementFn that this delegate's card is currently face down & in play
pub fn face_down_in_play<T>(game: &GameState, scope: Scope, _: &T) -> bool {
    let card = game.card(scope.card_id());
    card.is_face_down() && card.position().in_play()
}

/// RequirementFn that this delegate's card is currently in its owner's score
/// pile
pub fn scored_by_owner<T>(game: &GameState, scope: Scope, _: &T) -> bool {
    game.card(scope.card_id()).position() == CardPosition::Scored(scope.side())
}

/// A RequirementFn which restricts delegates to only listen to events for their
/// own card.
pub fn this_card(_game: &GameState, scope: Scope, card_id: &impl HasCardId) -> bool {
    scope.card_id() == card_id.card_id()
}

/// A RequirementFn which restricts delegates to only listen to events for their
/// own ability.
pub fn this_ability(_game: &GameState, scope: Scope, ability_id: &impl HasAbilityId) -> bool {
    scope.ability_id() == ability_id.ability_id()
}

/// A RequirementFn which checks if the current `raid_id` matches the stored
/// [RaidId] for this `scope`.
pub fn matching_raid<T>(game: &GameState, scope: Scope, _: &T) -> bool {
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
pub fn alert(game: &mut GameState, scope: Scope) {
    game.record_update(|| GameUpdate::AbilityTriggered(scope.ability_id()));
}

/// Invokes [alert] if the provided `number` is not zero.
pub fn alert_if_nonzero(game: &mut GameState, scope: Scope, number: u32) {
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

pub fn when_unveiled(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::UnveilProject(EventDelegate { requirement: this_card, mutation })
}

/// A delegate which triggers at dawn if a card is face up in play
pub fn at_dawn(mutation: MutationFn<TurnNumber>) -> Delegate {
    Delegate::Dawn(EventDelegate { requirement: face_up_in_play, mutation })
}

/// A delegate which triggers at dusk if a card is face up in play
pub fn at_dusk(mutation: MutationFn<TurnNumber>) -> Delegate {
    Delegate::Dusk(EventDelegate { requirement: face_up_in_play, mutation })
}

/// A minion delegate which triggers when it is encountered
pub fn on_encountered(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::EncounterMinion(EventDelegate { requirement: this_card, mutation })
}

/// Delegate to supply supplemental minion actions when encountered.
pub fn minion_combat_actions(
    transformation: TransformationFn<CardId, Vec<Option<CardPromptAction>>>,
) -> Delegate {
    Delegate::MinionCombatActions(QueryDelegate { requirement: this_card, transformation })
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

/// Delegate which fires when a weapon is used
pub fn on_weapon_used(
    requirement: RequirementFn<UsedWeapon>,
    mutation: MutationFn<UsedWeapon>,
) -> Delegate {
    Delegate::UsedWeapon(EventDelegate { requirement, mutation })
}

/// Delegate which fires when the 'access' phase of a raid begins.
pub fn on_raid_access_start(
    requirement: RequirementFn<RaidId>,
    mutation: MutationFn<RaidId>,
) -> Delegate {
    Delegate::RaidAccessStart(EventDelegate { requirement, mutation })
}

/// Delegate which fires when its card is accessed
pub fn on_accessed(mutation: MutationFn<CardId>) -> Delegate {
    Delegate::CardAccess(EventDelegate { requirement: this_card, mutation })
}

/// A delegate which fires when a raid ends in any way (except the game ending).
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

/// Delegate which transforms how a minion's health is calculated
pub fn on_calculate_health(transformation: TransformationFn<CardId, HealthValue>) -> Delegate {
    Delegate::HealthValue(QueryDelegate { requirement: this_card, transformation })
}

pub fn add_vault_access<const N: u32>(requirement: RequirementFn<RaidId>) -> Delegate {
    Delegate::VaultAccessCount(QueryDelegate {
        requirement,
        transformation: |_, _, _, current| current + N,
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

/// Helper to create a [CardStats] with the given [SchemePoints].
pub fn scheme_points(points: SchemePoints) -> CardStats {
    CardStats { scheme_points: Some(points), ..CardStats::default() }
}

/// Initiates a raid on the `target` room and stores the raid ID as ability
/// state.
pub fn initiate_raid(game: &mut GameState, scope: Scope, target: CardTarget) -> Result<()> {
    initiate_raid_with_callback(game, scope, target, |_, _| {})
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
) -> Result<()> {
    raid::core::initiate(game, target.room_id()?, InitiatedBy::Card, |game, raid_id| {
        game.ability_state_mut(scope.ability_id()).raid_id = Some(raid_id);
        on_begin(game, raid_id);
    })
}

/// Invokes `function` at most once per turn.
///
/// Stores ability state to track the last-invoked turn number
pub fn once_per_turn<T>(
    game: &mut GameState,
    scope: Scope,
    data: &T,
    function: MutationFn<T>,
) -> Result<()> {
    if utils::is_false(|| Some(game.ability_state(scope.ability_id())?.turn? == game.data.turn)) {
        save_turn(game, scope);
        function(game, scope, data)
    } else {
        Ok(())
    }
}

/// Stores the current turn as ability state for the provided `ability_id`.
pub fn save_turn(game: &mut GameState, ability_id: impl HasAbilityId) {
    game.ability_state_mut(ability_id.ability_id()).turn = Some(game.data.turn);
}

/// Helper to store the provided [RaidId] as ability state for this [Scope].
pub fn save_raid_id(
    game: &mut GameState,
    ability_id: impl HasAbilityId,
    raid_id: &RaidId,
) -> Result<()> {
    game.ability_state_mut(ability_id.ability_id()).raid_id = Some(*raid_id);
    Ok(())
}

/// Add `amount` to the stored mana in a card. Returns the new stored amount.
pub fn add_stored_mana(game: &mut GameState, card_id: CardId, amount: ManaValue) -> ManaValue {
    game.card_mut(card_id).data.stored_mana += amount;
    game.card(card_id).data.stored_mana
}

/// Creates a [SpecialEffects] to fire a given [Projectile].
pub fn projectile(projectile: Projectile) -> SpecialEffects {
    SpecialEffects { projectile: Some(projectile), additional_hit: None }
}

/// Delegate to attempt to unveil a project each turn at Dusk.
pub fn unveil_at_dusk() -> Delegate {
    Delegate::Dusk(EventDelegate {
        requirement: face_down_in_play,
        mutation: |g, s, _| mutations::try_unveil_project(g, s.card_id()).map(|_| ()),
    })
}

/// Delegate to store mana in a card when it is unveiled
pub fn store_mana_on_unveil<const N: u32>() -> Delegate {
    when_unveiled(|g, s, _| {
        add_stored_mana(g, s.card_id(), N);
        Ok(())
    })
}

/// Marks an ability as possible to activate while its card is face-down
pub fn activate_while_face_down() -> Delegate {
    Delegate::CanActivateWhileFaceDown(QueryDelegate {
        requirement: this_ability,
        transformation: |_g, _, _, current| current.with_override(true),
    })
}

/// Makes an ability's mana cost equal to the cost of its parent card while that
/// card is face-down.
pub fn face_down_ability_cost() -> Delegate {
    Delegate::AbilityManaCost(QueryDelegate {
        requirement: this_ability,
        transformation: |g, s, _, current| {
            if g.card(s.card_id()).is_face_up() {
                current
            } else {
                Some(current.unwrap_or(0) + queries::mana_cost(g, s.card_id())?)
            }
        },
    })
}

/// Sets the card prompt for the `side` player to show the provided non-`None`
/// `actions`.
pub fn set_card_prompt(
    game: &mut GameState,
    side: Side,
    actions: Vec<Option<CardPromptAction>>,
) -> Result<()> {
    mutations::set_prompt(
        game,
        side,
        SetPrompt::CardPrompt,
        GamePrompt::card_actions(actions.into_iter().flatten().collect()),
    )
}

/// A [CardPromptAction] for the `side` player to lose mana
pub fn lose_mana_prompt(
    game: &GameState,
    side: Side,
    amount: ActionCount,
) -> Option<CardPromptAction> {
    if mana::get(game, side, ManaPurpose::PayForTriggeredAbility) >= amount {
        Some(CardPromptAction::LoseMana(side, amount))
    } else {
        None
    }
}

/// A [CardPromptAction] for the `side` player to lose action points.
pub fn lose_actions_prompt(
    game: &GameState,
    side: Side,
    amount: ActionCount,
) -> Option<CardPromptAction> {
    if game.player(side).actions >= amount {
        Some(CardPromptAction::LoseActions(side, amount))
    } else {
        None
    }
}

/// A [CardPromptAction] for the `side` player to take damage if they are able
/// to without losing the game
pub fn take_damage_prompt(
    game: &GameState,
    ability_id: impl HasAbilityId,
    damage_type: DamageType,
    amount: u32,
) -> Option<CardPromptAction> {
    if game.hand(Side::Champion).count() >= amount as usize {
        Some(CardPromptAction::TakeDamage(ability_id.ability_id(), damage_type, amount))
    } else {
        None
    }
}
