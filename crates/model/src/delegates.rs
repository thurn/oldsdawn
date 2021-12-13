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

use crate::game::GameState;
use crate::primitives::{AbilityId, AttackValue, CardId, HealthValue, Side};
use std::fmt;
use std::fmt::{Debug, Formatter};
use strum_macros::EnumDiscriminants;

/// Context for which ability owns a delegate
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct Context {
    /// Side which owns this delegate.  
    side: Side,

    /// Ability which owns this delegate.
    ability_id: AbilityId,
}

impl Context {
    pub fn new(game: &GameState, ability_id: AbilityId) -> Self {
        Self { side: Side::Champion, ability_id }
    }

    pub fn side(&self) -> Side {
        self.side
    }

    pub fn ability_id(&self) -> AbilityId {
        self.ability_id
    }

    pub fn card_id(&self) -> CardId {
        self.ability_id.card_id
    }
}

pub type RequirementFn<T> = fn(&GameState, Context, T) -> bool;
pub type MutationFn<T> = fn(&mut GameState, Context, T);
pub type TransformationFn<T, R> = fn(&GameState, Context, T, R) -> R;

#[derive(Clone)]
pub struct EventDelegate<T> {
    pub requirement: RequirementFn<T>,
    pub mutation: MutationFn<T>,
}

#[derive(Clone)]
pub struct QueryDelegate<T, R> {
    pub requirement: RequirementFn<T>,
    pub transformation: TransformationFn<T, R>,
}

#[derive(Clone, EnumDiscriminants)]
pub enum Delegate {
    /// Event when either player draws a card
    OnDrawCard(EventDelegate<CardId>),
    /// Event when either player plays a card
    OnPlayCard(EventDelegate<CardId>),
    /// Event when a raid ability is activated
    OnActivateRaidAbility(EventDelegate<AbilityId>),

    /// Query whether a given card can currently be played
    CanPlayCard(QueryDelegate<CardId, bool>),

    /// Query the current attack value of a card
    GetAttackValue(QueryDelegate<CardId, AttackValue>),
    /// Query the current health value of a card
    GetHealthValue(QueryDelegate<CardId, HealthValue>),
}

impl Debug for Delegate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Delegate::{:?}", DelegateDiscriminants::from(self))
    }
}

pub fn on_draw_card(game: &mut GameState, context: Context, delegate: &Delegate, data: CardId) {
    match delegate {
        Delegate::OnDrawCard(EventDelegate { requirement, mutation })
            if requirement(game, context, data) =>
        {
            mutation(game, context, data)
        }
        _ => (),
    }
}

pub fn on_play_card(game: &mut GameState, context: Context, delegate: &Delegate, data: CardId) {
    match delegate {
        Delegate::OnPlayCard(EventDelegate { requirement, mutation })
            if requirement(game, context, data) =>
        {
            mutation(game, context, data)
        }
        _ => (),
    }
}

pub fn can_play_card(
    game: &GameState,
    context: Context,
    delegate: &Delegate,
    data: CardId,
    current: bool,
) -> bool {
    match delegate {
        Delegate::CanPlayCard(QueryDelegate { requirement, transformation })
            if requirement(game, context, data) =>
        {
            transformation(game, context, data, current)
        }
        _ => current,
    }
}

pub fn get_attack_value(
    game: &GameState,
    context: Context,
    delegate: &Delegate,
    data: CardId,
    current: AttackValue,
) -> AttackValue {
    match delegate {
        Delegate::GetAttackValue(QueryDelegate { requirement, transformation })
            if requirement(game, context, data) =>
        {
            transformation(game, context, data, current)
        }
        _ => current,
    }
}

pub fn get_health_value(
    game: &GameState,
    context: Context,
    delegate: &Delegate,
    data: CardId,
    current: HealthValue,
) -> HealthValue {
    match delegate {
        Delegate::GetHealthValue(QueryDelegate { requirement, transformation })
            if requirement(game, context, data) =>
        {
            transformation(game, context, data, current)
        }
        _ => current,
    }
}
