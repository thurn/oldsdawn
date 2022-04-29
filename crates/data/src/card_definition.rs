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

//! Data structures for defining card rules -- the parts of a card which do not
//! vary from game to game.

#![allow(clippy::use_self)] // Required to use EnumKind

use std::fmt::Debug;

use enum_kinds::EnumKind;

use crate::card_name::CardName;
use crate::delegates::Delegate;
use crate::primitives::{
    AbilityId, AbilityIndex, ActionCount, AttackValue, CardId, CardSubtype, CardType, Faction,
    HealthValue, LevelValue, ManaValue, PointsValue, Rarity, RoomId, School, ShieldValue, Side,
    Sprite,
};
use crate::special_effects::{Projectile, TimedEffect};
use crate::text::AbilityText;

/// Cost to play a card or activate an ability
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct Cost {
    /// Cost in mana
    pub mana: Option<ManaValue>,
    /// Cost in action points
    pub actions: ActionCount,
}

impl Default for Cost {
    fn default() -> Self {
        Self { mana: None, actions: 1 }
    }
}

/// An activated ability used by Weapons to increase their attack value by
/// paying a mana cost during a raid encounter. Can be used any number of times.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct AttackBoost {
    /// Cost to activate an instance of this boost
    pub cost: ManaValue,
    /// Bonus to attack added for each activation
    pub bonus: AttackValue,
}

/// Scoring information about a card
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct SchemePoints {
    /// Required number of level counters to score this card
    pub level_requirement: LevelValue,
    /// Number of points received for scoring this card
    pub points: PointsValue,
}

/// Base card state values
#[derive(PartialEq, Eq, Hash, Debug, Clone, Default)]
pub struct CardStats {
    /// Damage required to destroy this card
    pub health: Option<HealthValue>,
    /// Mana cost required in order to interact with this card
    pub shield: Option<ShieldValue>,
    /// Base damage dealt by this card during an encounter
    pub base_attack: Option<AttackValue>,
    /// An increase in base attack damage for a fixed cost which an ability can
    /// apply to this card
    pub attack_boost: Option<AttackBoost>,
    /// Level Requirement & points for scoring this card
    pub scheme_points: Option<SchemePoints>,
    /// Can this card gain levels from the 'level up room' action?
    pub can_level_up: bool,
}

/// Describes how ability being triggered should be communicated in the UI
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum TriggerIndicator {
    /// Display a visual alert that an ability has triggered.
    Alert,

    /// Apply ability's effects without a visual effect
    Silent,
}

/// Possible types of ability
#[derive(PartialEq, Eq, Hash, Debug, Clone, EnumKind)]
#[enum_kind(AbilityTypeKind)]
pub enum AbilityType {
    /// Standard abilities function at all times without requiring activation.
    Standard(TriggerIndicator),

    /// Encounter abilities can be activated by the Champion during a raid
    /// encounter, typically in order to increase a Weapon's attack.
    Encounter,

    /// Activated abilities have an associated cost in order to be used.
    Activated(Cost),
}

/// Abilities are the unit of action in Spelldawn. Their behavior is provided by
/// the Delegate system, see delegates.rs for more information.
#[derive(Debug)]
pub struct Ability {
    pub text: AbilityText,
    pub ability_type: AbilityType,
    pub delegates: Vec<Delegate>,
}

/// Describes custom visual & audio effects for this card
#[derive(Debug, Default)]
pub struct SpecialEffects {
    /// Projectile to be fired by this card during targeted interactions
    pub projectile: Option<Projectile>,
    /// Additional hit effect after primary projectile impact
    pub additional_hit: Option<TimedEffect>,
}

pub type RoomPredicate = fn(RoomId) -> bool;

/// Allows cards to provide special targeting behavior beyond what is normal for
/// their [CardType].
#[derive(Debug)]
pub enum CustomTargeting {
    /// Target a specific room when played. Only rooms for which the provided
    /// [RoomPredicate] returns true are considered valid targets.
    TargetRoom(RoomPredicate),
}

/// Individual card configuration; properties which are not universal for all
/// cards
#[derive(Debug, Default)]
pub struct CardConfig {
    pub stats: CardStats,
    pub faction: Option<Faction>,
    pub subtypes: Vec<CardSubtype>,
    pub custom_targeting: Option<CustomTargeting>,
    pub special_effects: SpecialEffects,
}

/// The fundamental object defining the behavior of a given card in Spelldawn
///
/// This struct's top-level fields should be universal properties which need to
/// be set by every card
#[derive(Debug)]
pub struct CardDefinition {
    pub name: CardName,
    pub cost: Cost,
    pub image: Sprite,
    pub card_type: CardType,
    pub side: Side,
    pub school: School,
    pub rarity: Rarity,
    pub abilities: Vec<Ability>,
    pub config: CardConfig,
}

impl CardDefinition {
    /// Returns the ability at the given index. Panics if no ability with this
    /// index exists.
    pub fn ability(&self, index: AbilityIndex) -> &Ability {
        &self.abilities[index.value()]
    }

    pub fn ability_ids(&self, card_id: CardId) -> impl Iterator<Item = AbilityId> {
        (0..self.abilities.len()).map(move |i| AbilityId::new(card_id, i))
    }
}
