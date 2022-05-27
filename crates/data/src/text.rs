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

#![allow(clippy::use_self)] // Required to use EnumKind

use std::fmt;
use std::fmt::{Debug, Formatter};

use enum_kinds::EnumKind;

use crate::card_definition::Cost;
use crate::delegates::Scope;
use crate::game::GameState;
use crate::primitives::{ActionCount, BreachValue, DamageType, ManaValue};

/// Text describing what an ability does. Can be a function (if text is dynamic)
/// or a vector of [TextToken]s.
#[derive(Clone)]
pub enum AbilityText {
    Text(Vec<TextToken>),
    TextFn(TextFn),
}

impl Debug for AbilityText {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            AbilityText::Text(tokens) => write!(f, "{:?}", tokens),
            AbilityText::TextFn(_) => write!(f, "<TextFn>"),
        }
    }
}

/// Different types of text which can appear in rules text
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum TextToken {
    Literal(String),
    Number(NumericOperator, u32),
    Mana(ManaValue),
    Actions(ActionCount),
    Keyword(Keyword),
    Reminder(String),
    Cost(Vec<Self>),
}

/// A function which produces rules text
pub type TextFn = fn(&GameState, Scope) -> Vec<TextToken>;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
/// Location of a keyword within a sentence, used to determine capitalization
pub enum Sentence {
    Start,
    Internal,
}

/// Identifies a keyword or concept which appears in rules text
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, EnumKind)]
#[enum_kind(KeywordKind, derive(Ord, PartialOrd))]
pub enum Keyword {
    Play,
    Dawn,
    Dusk,
    Score,
    Combat,
    Encounter,
    Unveil,
    SuccessfulRaid,
    Store(Sentence, u32),
    Take(Sentence, u32),
    DealDamage(u32, DamageType),
    InnerRoom(Sentence),
    EndRaid,
    Breach(BreachValue),
    LevelUp,
    Trap,
    Construct,
}

impl Keyword {
    pub fn kind(&self) -> KeywordKind {
        self.into()
    }
}

/// A symbol applied to a number which appears in rules text
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum NumericOperator {
    None,
    Add,
}

impl From<&str> for TextToken {
    fn from(s: &str) -> Self {
        Self::Literal(s.to_owned())
    }
}

impl From<u32> for TextToken {
    fn from(v: u32) -> Self {
        Self::Number(NumericOperator::None, v)
    }
}

impl From<Keyword> for TextToken {
    fn from(k: Keyword) -> Self {
        Self::Keyword(k)
    }
}

impl<T> From<Cost<T>> for TextToken {
    fn from(cost: Cost<T>) -> Self {
        let mut result = vec![];
        if let Some(mana) = cost.mana {
            result.push(Self::Mana(mana))
        }

        if cost.actions > 1 {
            result.push(Self::Actions(cost.actions));
        }

        Self::Cost(result)
    }
}
