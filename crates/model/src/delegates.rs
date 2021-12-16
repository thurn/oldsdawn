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

use crate::card_definition::CardStats;
use crate::card_state::{CardData, CardPosition};
use crate::game::{GameState, RaidState};
use crate::primitives::{
    AbilityId, AttackValue, BoostCount, BoostData, CardId, HealthValue, RaidId, Side, TurnNumber,
};
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::sync::Arc;
use strum_macros::EnumDiscriminants;

/// Scope for which ability owns a delegate
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct Scope {
    /// Ability which owns this delegate.
    ability_id: AbilityId,
}

impl From<Scope> for CardId {
    fn from(scope: Scope) -> Self {
        scope.ability_id.into()
    }
}

impl Scope {
    pub fn new(ability_id: AbilityId) -> Self {
        Self { ability_id }
    }

    pub fn side(&self) -> Side {
        self.card_id().side
    }

    pub fn ability_id(&self) -> AbilityId {
        self.ability_id
    }

    pub fn card_id(&self) -> CardId {
        self.ability_id.card_id
    }
}

pub type RequirementFn<T> = fn(&GameState, Scope, T) -> bool;
pub type MutationFn<T> = fn(&mut GameState, Scope, T);
pub type TransformationFn<T, R> = fn(&GameState, Scope, T, R) -> R;

// pub type RequirementFn<T> = Arc<dyn Fn(&GameState, Scope, T) -> bool>;
// pub type MutationFn<T> = Arc<dyn Fn(&mut GameState, Scope, T)>;
// pub type TransformationFn<T, R> = Arc<dyn Fn(&GameState, Scope, T, R) -> R>;

pub struct EventDelegate<T> {
    pub requirement: RequirementFn<T>,
    pub mutation: MutationFn<T>,
}

impl<T> EventDelegate<T> {
    pub fn new(requirement: RequirementFn<T>, mutation: MutationFn<T>) -> Self {
        EventDelegate { requirement, mutation }
    }
}

pub struct QueryDelegate<T, R> {
    pub requirement: RequirementFn<T>,
    pub transformation: TransformationFn<T, R>,
}

impl<T, R> QueryDelegate<T, R> {
    pub fn new(requirement: RequirementFn<T>, transformation: TransformationFn<T, R>) -> Self {
        QueryDelegate { requirement, transformation }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct CardMoved {
    pub old_position: CardPosition,
    pub new_position: CardPosition,
}

#[derive(EnumDiscriminants)]
pub enum Delegate {
    /// The Champion's turn begins
    OnDawn(EventDelegate<TurnNumber>),
    /// The Overlord's turn begins
    OnDusk(EventDelegate<TurnNumber>),
    /// A card is moved from a Deck position to a Hand position
    OnDrawCard(EventDelegate<CardId>),
    /// A card is moved from a Hand position to an Arena position *or* explicitly played via the
    /// 'play' action
    OnPlayCard(EventDelegate<CardId>),
    /// A card is moved to a new position
    OnMoveCard(EventDelegate<CardMoved>),
    /// A card is scored by the Overlord
    OnScoreScheme(EventDelegate<CardId>),
    OnStealScheme(EventDelegate<CardId>),

    /// A Raid is initiated
    OnRaidBegin(EventDelegate<RaidState>),
    /// A minion is encountered during a raid
    OnEncounterBegin(EventDelegate<RaidState>),
    /// A weapon boost is activated for a given card
    OnActivateBoost(EventDelegate<BoostData>),
    /// A minion is defeated during an encounter by dealing damage to it equal to its health
    OnMinionDefeated(EventDelegate<CardId>),
    /// A minion's 'combat' ability is triggered during an encounter, typically because the minion
    /// was not defeated by the Champion.
    OnMinionCombatAbility(EventDelegate<CardId>),
    /// A minion finishes being encountered during a raid. Invokes regardless of whether the
    /// encounter was successful.
    OnEncounterEnd(EventDelegate<RaidState>),
    /// A Raid is completed, either successfully or unsuccessfully.
    OnRaidEnd(EventDelegate<RaidState>),

    /// Stored mana is taken from a card
    OnStoredManaTaken(EventDelegate<CardId>),

    /// Query whether a given card can currently be played
    CanPlayCard(QueryDelegate<CardId, bool>),

    /// Query the current attack value of a card. Invoked with [CardStats::base_attack] or 0.
    GetAttackValue(QueryDelegate<CardId, AttackValue>),
    /// Query the current health value of a card. Invoked with [CardStats::health] or 0.
    GetHealthValue(QueryDelegate<CardId, HealthValue>),
    /// Get the current boost count of a card. Invoked with the value of [CardData::boost_count].
    GetBoostCount(QueryDelegate<CardId, BoostCount>),
}

impl Debug for Delegate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Delegate::{:?}", DelegateDiscriminants::from(self))
    }
}

pub fn on_dawn(game: &mut GameState, scope: Scope, delegate: &Delegate, data: TurnNumber) {
    match delegate {
        Delegate::OnDawn(EventDelegate { requirement, mutation })
            if requirement(game, scope, data) =>
        {
            mutation(game, scope, data)
        }
        _ => (),
    }
}

pub fn on_dusk(game: &mut GameState, scope: Scope, delegate: &Delegate, data: TurnNumber) {
    match delegate {
        Delegate::OnDusk(EventDelegate { requirement, mutation })
            if requirement(game, scope, data) =>
        {
            mutation(game, scope, data)
        }
        _ => (),
    }
}

pub fn on_draw_card(game: &mut GameState, scope: Scope, delegate: &Delegate, data: CardId) {
    match delegate {
        Delegate::OnDrawCard(EventDelegate { requirement, mutation })
            if requirement(game, scope, data) =>
        {
            mutation(game, scope, data)
        }
        _ => (),
    }
}

pub fn on_play_card(game: &mut GameState, scope: Scope, delegate: &Delegate, data: CardId) {
    match delegate {
        Delegate::OnPlayCard(EventDelegate { requirement, mutation })
            if requirement(game, scope, data) =>
        {
            mutation(game, scope, data)
        }
        _ => (),
    }
}

pub fn on_move_card(game: &mut GameState, scope: Scope, delegate: &Delegate, data: CardMoved) {
    match delegate {
        Delegate::OnMoveCard(EventDelegate { requirement, mutation })
            if requirement(game, scope, data) =>
        {
            mutation(game, scope, data)
        }
        _ => (),
    }
}

pub fn on_score_scheme(game: &mut GameState, scope: Scope, delegate: &Delegate, data: CardId) {
    match delegate {
        Delegate::OnScoreScheme(EventDelegate { requirement, mutation })
            if requirement(game, scope, data) =>
        {
            mutation(game, scope, data)
        }
        _ => (),
    }
}

pub fn on_minion_combat_ability(
    game: &mut GameState,
    scope: Scope,
    delegate: &Delegate,
    data: CardId,
) {
    match delegate {
        Delegate::OnMinionCombatAbility(EventDelegate { requirement, mutation })
            if requirement(game, scope, data) =>
        {
            mutation(game, scope, data)
        }
        _ => (),
    }
}

pub fn on_activate_boost(game: &mut GameState, scope: Scope, delegate: &Delegate, data: BoostData) {
    match delegate {
        Delegate::OnActivateBoost(EventDelegate { requirement, mutation })
            if requirement(game, scope, data) =>
        {
            mutation(game, scope, data)
        }
        _ => (),
    }
}

pub fn on_stored_mana_taken(game: &mut GameState, scope: Scope, delegate: &Delegate, data: CardId) {
    match delegate {
        Delegate::OnStoredManaTaken(EventDelegate { requirement, mutation })
            if requirement(game, scope, data) =>
        {
            mutation(game, scope, data)
        }
        _ => (),
    }
}

pub fn on_raid_begin(game: &mut GameState, scope: Scope, delegate: &Delegate, data: RaidState) {
    match delegate {
        Delegate::OnRaidBegin(EventDelegate { requirement, mutation })
            if requirement(game, scope, data) =>
        {
            mutation(game, scope, data)
        }
        _ => (),
    }
}

pub fn on_raid_end(game: &mut GameState, scope: Scope, delegate: &Delegate, data: RaidState) {
    match delegate {
        Delegate::OnRaidEnd(EventDelegate { requirement, mutation })
            if requirement(game, scope, data) =>
        {
            mutation(game, scope, data)
        }
        _ => (),
    }
}

pub fn can_play_card(
    game: &GameState,
    scope: Scope,
    delegate: &Delegate,
    data: CardId,
    current: bool,
) -> bool {
    match delegate {
        Delegate::CanPlayCard(QueryDelegate { requirement, transformation })
            if requirement(game, scope, data) =>
        {
            transformation(game, scope, data, current)
        }
        _ => current,
    }
}

pub fn get_attack_value(
    game: &GameState,
    scope: Scope,
    delegate: &Delegate,
    data: CardId,
    current: AttackValue,
) -> AttackValue {
    match delegate {
        Delegate::GetAttackValue(QueryDelegate { requirement, transformation })
            if requirement(game, scope, data) =>
        {
            transformation(game, scope, data, current)
        }
        _ => current,
    }
}

pub fn get_health_value(
    game: &GameState,
    scope: Scope,
    delegate: &Delegate,
    data: CardId,
    current: HealthValue,
) -> HealthValue {
    match delegate {
        Delegate::GetHealthValue(QueryDelegate { requirement, transformation })
            if requirement(game, scope, data) =>
        {
            transformation(game, scope, data, current)
        }
        _ => current,
    }
}

pub fn get_boost_count(
    game: &GameState,
    scope: Scope,
    delegate: &Delegate,
    data: CardId,
    current: BoostCount,
) -> BoostCount {
    match delegate {
        Delegate::GetBoostCount(QueryDelegate { requirement, transformation })
            if requirement(game, scope, data) =>
        {
            transformation(game, scope, data, current)
        }
        _ => current,
    }
}
