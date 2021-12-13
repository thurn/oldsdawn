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
use crate::delegates::Delegate;
use crate::primitives::{
    ActionCount, AttackValue, CardSubtype, CardType, HealthValue, ManaValue, Rarity, School,
    ShieldValue, Side, SpriteAddress,
};
use std::fmt::Debug;

/// Cost to play a card or activate an ability
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Cost {
    pub mana: ManaValue,
    pub actions: ActionCount,
}

/// An activated ability used by Weapons to increase their attack value by paying a mana cost during
/// a raid encounter. Can be used any number of times. By default, attack bonuses last for the
/// duration of the current encounter only.
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct PowerUp {
    pub cost: ManaValue,
    pub bonus: AttackValue,
}

/// Base card numeric values
#[derive(PartialEq, Eq, Hash, Debug, Clone, Default)]
pub struct CardStats {
    pub health: Option<HealthValue>,
    pub shield: Option<ShieldValue>,
    pub base_attack: Option<AttackValue>,
    pub power_up: Option<PowerUp>,
}

/// Possible types of ability
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum AbilityType {
    /// Standard abilities function at all times without requiring activation
    Standard,

    /// Activated abilities have an associated cost in order to be used
    Activated(Cost),
}

/// Text describing what an ability does
#[derive(PartialEq, Eq, Hash, Debug, Clone, Default)]
pub struct CardText {
    pub text: String,
}

/// Abilities are the unit of action in Spelldawn. Their behavior is provided by the Delegate
/// system, see delegates.rs for more information.
#[derive(Debug, Clone)]
pub struct Ability {
    pub text: CardText,
    pub ability_type: AbilityType,
    pub delegates: Vec<Delegate>,
}

/// Individual card configuration; properties which are not universal for all cards
#[derive(Debug, Default, Clone)]
pub struct CardConfig {
    pub stats: CardStats,
    pub subtypes: Vec<CardSubtype>,
}

/// The fundamental object defining the behavior of a given card in Spelldawn
///
/// This struct's top-level fields should be universal properties which apply to every card
#[derive(Debug, Clone)]
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
