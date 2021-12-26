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

use std::fmt;
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::sync::Arc;

use macros::DelegateEnum;
use strum_macros::EnumDiscriminants;
use tracing::{info_span, Span};

use crate::card_definition::{CardStats, Cost};
use crate::card_state::{CardData, CardPosition};
use crate::game::{GameState, RaidState};
use crate::primitives::{
    AbilityId, ActionCount, AttackValue, BoostCount, BoostData, CardId, HealthValue, ManaValue,
    RaidId, ShieldValue, Side, TurnNumber,
};

/// Scope for which ability owns a delegate
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
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

impl fmt::Debug for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.ability_id)
    }
}

pub type RequirementFn<T> = fn(&GameState, Scope, T) -> bool;
pub type MutationFn<T> = fn(&mut GameState, Scope, T);
pub type TransformationFn<T, R> = fn(&GameState, Scope, T, R) -> R;

#[derive(Copy, Clone)]
pub struct EventDelegate<T> {
    pub requirement: RequirementFn<T>,
    pub mutation: MutationFn<T>,
}

impl<T> EventDelegate<T> {
    pub fn new(requirement: RequirementFn<T>, mutation: MutationFn<T>) -> Self {
        EventDelegate { requirement, mutation }
    }
}

#[derive(Copy, Clone)]
pub struct QueryDelegate<T, R> {
    pub requirement: RequirementFn<T>,
    pub transformation: TransformationFn<T, R>,
}

impl<T, R> QueryDelegate<T, R> {
    pub fn new(requirement: RequirementFn<T>, transformation: TransformationFn<T, R>) -> Self {
        QueryDelegate { requirement, transformation }
    }
}

/// A Flag is a variant of boolean which typically indicates whether some game action can currently
/// be taken. Flags have a 'default' state, which is the value of the flag based on standard game
/// rules, and an 'override' state, which is a value set by specific delegates. An override of
/// 'false' takes precedence over an override of 'true'.
///
/// For example, the 'CanPlay' delegate will be invoked with Flag::Default(false) if a card cannot
/// currently be played according to the standard game rules (sufficient mana available, correct
/// player's turn, etc). A delegate could transform this via `with_override(true)` to allow the
/// card to be played. A second delegate could set `with_override(false)` to prevent the card from
/// being played, and this would take priority.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum Flag {
    Default(bool),
    Override(bool),
}

impl Flag {
    pub fn new(value: bool) -> Self {
        Flag::Default(value)
    }

    /// Incorporates an override into this flag, following the precedence rules
    /// described above
    pub fn with_override(self, value: bool) -> Self {
        match self {
            Self::Default(current) => Self::Override(value),
            Self::Override(current) => Self::Override(current && value),
        }
    }
}

impl From<Flag> for bool {
    fn from(flag: Flag) -> Self {
        match flag {
            Flag::Default(value) | Flag::Override(value) => value,
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct CardMoved {
    pub old_position: CardPosition,
    pub new_position: CardPosition,
}

#[derive(EnumDiscriminants, DelegateEnum)]
#[strum_discriminants(name(DelegateKind))]
pub enum Delegate {
    /// The Champion's turn begins
    Dawn(EventDelegate<TurnNumber>),
    /// The Overlord's turn begins
    Dusk(EventDelegate<TurnNumber>),
    /// A card is moved from a Deck position to a Hand position
    DrawCard(EventDelegate<CardId>),
    /// A card has been selected to play via the Play action and should have
    /// additional costs deducted.
    PayCardCosts(EventDelegate<CardId>),
    /// A card has been played via the Play action and has had its costs paid
    CastCard(EventDelegate<CardId>),
    /// A card is moved from a non-arena position to an arena position
    PlayCard(EventDelegate<CardId>),
    /// A card is moved to a new position
    MoveCard(EventDelegate<CardMoved>),
    /// A card is scored by the Overlord
    ScoreScheme(EventDelegate<CardId>),
    /// A card is scored by the Champion
    StealScheme(EventDelegate<CardId>),
    /// A Raid is initiated
    RaidBegin(EventDelegate<RaidState>),
    /// A minion is encountered during a raid
    EncounterBegin(EventDelegate<RaidState>),
    /// A weapon boost is activated for a given card
    ActivateBoost(EventDelegate<BoostData>),
    /// A minion is defeated during an encounter by dealing damage to it equal
    /// to its health
    MinionDefeated(EventDelegate<CardId>),
    /// A minion's 'combat' ability is triggered during an encounter, typically
    /// because the minion was not defeated by the Champion.
    MinionCombatAbility(EventDelegate<CardId>),
    /// A minion finishes being encountered during a raid. Invokes regardless of
    /// whether the encounter was successful.
    EncounterEnd(EventDelegate<RaidState>),
    /// A Raid is completed, either successfully or unsuccessfully.
    RaidEnd(EventDelegate<RaidState>),
    /// Stored mana is taken from a card
    StoredManaTaken(EventDelegate<CardId>),

    /// Query whether a given card can currently be played.
    CanPlayCard(QueryDelegate<CardId, Flag>),

    /// Query the current mana cost of a card. Invoked with [Cost::mana].
    ManaCost(QueryDelegate<CardId, Option<ManaValue>>),
    /// Query the current mana cost of a card. Invoked with [Cost::actions].
    ActionCost(QueryDelegate<CardId, ActionCount>),
    /// Query the current attack value of a card. Invoked with
    /// [CardStats::base_attack] or 0.
    AttackValue(QueryDelegate<CardId, AttackValue>),
    /// Query the current health value of a card. Invoked with
    /// [CardStats::health] or 0.
    HealthValue(QueryDelegate<CardId, HealthValue>),
    /// Query the current shield value of a card. Invoked with
    /// [CardStats::shield] or 0.
    ShieldValue(QueryDelegate<CardId, ShieldValue>),
    /// Get the current boost count of a card. Invoked with the value of
    /// [CardData::boost_count].
    BoostCount(QueryDelegate<CardId, BoostCount>),
}

impl fmt::Debug for Delegate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Delegate::{:?}", DelegateKind::from(self))
    }
}

pub trait EventData<T: fmt::Debug>: fmt::Debug {
    fn data(&self) -> T;

    fn get(delegate: &Delegate) -> Option<EventDelegate<T>>;
}

pub trait QueryData<TData: fmt::Debug, TResult: fmt::Debug>: fmt::Debug {
    fn data(&self) -> TData;

    fn get(delegate: &Delegate) -> Option<QueryDelegate<TData, TResult>>;
}

/*

Example of the code generated in this file:

#[derive(Debug, Copy, Clone)]
pub struct OnDawnEvent(pub TurnNumber);

impl EventData<TurnNumber> for OnDawnEvent {
    fn data(&self) -> TurnNumber {
        self.0
    }

    fn get(delegate: &Delegate) -> Option<EventDelegate<TurnNumber>> {
        match delegate {
            Delegate::OnDawn(d) => Some(*d),
            _ => None,
        }
    }

    fn span(&self) -> Span {
        let data = self.data();
        info_span!("on_dawn", ?data)
    }
}

*/
