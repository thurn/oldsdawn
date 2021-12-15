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

use crate::card_name::CardName;
use crate::delegates::{Delegate, Scope};
use crate::game::GameState;
use crate::primitives::{
    ActionCount, AttackValue, CardSubtype, CardType, HealthValue, ManaValue, Rarity, School,
    ShieldValue, Side, SpriteAddress,
};
use std::fmt;
use std::fmt::{Debug, Formatter};

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum Keyword {
    Play,
    Dawn,
    Dusk,
    Store,
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum NumericOperator {
    None,
    Add,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum TextToken {
    Literal(String),
    Number(NumericOperator, u32),
    Mana(ManaValue),
    Keyword(Keyword),
    Cost(Vec<TextToken>),
}

pub type TextFn = fn(&GameState, Scope) -> Vec<TextToken>;

/// Text describing what an ability does
#[derive(Clone)]
pub enum CardText {
    Text(Vec<TextToken>),
    TextFn(TextFn),
}

impl Debug for CardText {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CardText::Text(tokens) => write!(f, "{:?}", tokens),
            CardText::TextFn(_) => write!(f, "<TextFn>"),
        }
    }
}

/// Cost to play a card or activate an ability
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Cost {
    pub mana: ManaValue,
    pub actions: ActionCount,
}

/// An activated ability used by Weapons to increase their attack value by paying a mana cost during
/// a raid encounter. Can be used any number of times.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct AttackBoost {
    pub cost: ManaValue,
    pub bonus: AttackValue,
}

/// Base card numeric values
#[derive(PartialEq, Eq, Hash, Debug, Clone, Default)]
pub struct CardStats {
    /// Damage required to destroy this card
    pub health: Option<HealthValue>,
    /// Mana cost required in order to attack this card
    pub shield: Option<ShieldValue>,
    /// Base damage dealt by this card during an encounter
    pub base_attack: Option<AttackValue>,
    /// An increase in base attack damage for a fixed cost which an ability can apply to this card
    pub attack_boost: Option<AttackBoost>,
    /// An amount of mana which an ability can store in this card
    pub store_mana: Option<ManaValue>,
}

/// Possible types of ability
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum AbilityType {
    /// Standard abilities function at all times without requiring activation.
    Standard,

    /// Encounter abilities can be activated by the Champion during a raid encounter, typically
    /// in order to increase a Weapon's attack.  
    Encounter,

    /// Activated abilities have an associated cost in order to be used.
    Activated(Cost),
}

/// Abilities are the unit of action in Spelldawn. Their behavior is provided by the Delegate
/// system, see delegates.rs for more information.
#[derive(Debug)]
pub struct Ability {
    pub text: CardText,
    pub ability_type: AbilityType,
    pub delegates: Vec<Delegate>,
}

/// Individual card configuration; properties which are not universal for all cards
#[derive(Debug, Default)]
pub struct CardConfig {
    pub stats: CardStats,
    pub subtypes: Vec<CardSubtype>,
}

/// The fundamental object defining the behavior of a given card in Spelldawn
///
/// This struct's top-level fields should be universal properties which apply to every card
#[derive(Debug)]
pub struct CardDefinition {
    pub name: CardName,
    pub cost: Cost,
    pub image: SpriteAddress,
    pub card_type: CardType,
    pub side: Side,
    pub school: School,
    pub rarity: Rarity,
    pub abilities: Vec<Ability>,
    pub config: CardConfig,
}
