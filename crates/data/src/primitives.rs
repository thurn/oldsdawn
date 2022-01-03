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

//! Fundamental types and data structures for Spelldawn

use std::fmt;

use serde::{Deserialize, Serialize};

pub type TurnNumber = u32;
pub type ActionCount = u32;
pub type ManaValue = u32;
pub type PointsValue = u32;
pub type HealthValue = u32;
pub type AttackValue = u32;
pub type ShieldValue = u32;
pub type BoostCount = u32;
pub type LevelValue = u32;

/// Identifies a player across different games
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct PlayerId {
    pub value: u64,
}

impl PlayerId {
    pub fn new(value: u64) -> Self {
        Self { value }
    }

    /// Byte array representation of this ID
    pub fn key(&self) -> [u8; 8] {
        self.value.to_be_bytes()
    }
}

impl fmt::Debug for PlayerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Identifies an ongoing game
#[derive(Copy, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct GameId {
    pub value: u64,
}

impl GameId {
    pub fn new(value: u64) -> Self {
        Self { value }
    }

    /// Byte array representation of this ID
    pub fn key(&self) -> [u8; 8] {
        self.value.to_be_bytes()
    }
}

impl fmt::Debug for GameId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// The two players in a game: Overlord & Champion
#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub enum Side {
    Overlord,
    Champion,
}

impl Side {
    /// Gets the opponent of the provided player
    pub fn opponent(&self) -> Self {
        match self {
            Side::Champion => Self::Overlord,
            Side::Overlord => Self::Champion,
        }
    }
}

impl fmt::Debug for Side {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Side::Overlord => "Overlord",
                Side::Champion => "Champion",
            }
        )
    }
}

/// Identifies a card in an ongoing game
#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct CardId {
    pub side: Side,
    pub index: usize,
}

impl CardId {
    pub fn new(side: Side, index: usize) -> Self {
        Self { side, index }
    }
}

impl fmt::Debug for CardId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}",
            match self.side {
                Side::Overlord => "O",
                Side::Champion => "C",
            },
            self.index
        )
    }
}

/// Identifies an ability position within a card's 'abilities' vector
#[derive(PartialEq, Eq, Hash, Copy, Clone, Ord, PartialOrd, Serialize, Deserialize)]
pub struct AbilityIndex(pub usize);

impl fmt::Debug for AbilityIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifies an ability within a card. Abilities are the only game entity
/// which may contain delegates..
#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct AbilityId {
    pub card_id: CardId,
    pub index: AbilityIndex,
}

impl fmt::Debug for AbilityId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}[{:?}]", self.card_id, self.index)
    }
}

impl AbilityId {
    pub fn new(card_id: CardId, index: usize) -> Self {
        Self { card_id, index: AbilityIndex(index) }
    }
}

/// Uniquely identifies a raid within a given game
#[derive(PartialEq, Eq, Hash, Copy, Clone, Serialize, Deserialize)]
pub struct RaidId(pub u32);

impl fmt::Debug for RaidId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

/// Contains the URL of an image asset within a game
#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub struct Sprite {
    pub address: String,
}

impl Sprite {
    pub fn new(address: impl Into<String>) -> Self {
        Self { address: address.into() }
    }
}

/// The schools of magic, which provide restrictions on players during
/// deckbuilding
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum School {
    Neutral,
    Shadow,
    Nature,
    Time,
}

/// The possible Rooms in which the Overlord player may play their cards.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum RoomId {
    /// The Overlord's deck
    Vault,
    /// The Overlord's hand
    Sanctum,
    /// The Overlord's discard pile
    Crypts,
    RoomA,
    RoomB,
    RoomC,
    RoomD,
    RoomE,
}

impl RoomId {
    /// An 'inner room' is one of the three predefined rooms for the Overlord
    /// player's deck, hand, and discard pile. Inner rooms cannot contain
    /// Schemes or Projects.
    pub fn is_inner_room(&self) -> bool {
        matches!(self, RoomId::Vault | RoomId::Sanctum | RoomId::Crypts)
    }
}

/// Used to control where a card is rendered within a room
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum RoomLocation {
    Defender,
    Occupant,
}

/// Used to control where an item is rendered within the Champion's item display
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ItemLocation {
    Weapons,
    Artifacts,
}

/// The three possible factions of weapons and minions. Minions can only be
/// damaged by weapons from the same faction.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Faction {
    Mortal,
    Abyssal,
    Infernal,
}

/// Rarity of a card, used to determine how likely it is to appear in randomized
/// rewards.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,

    /// Card cannot be obtained via random rewards
    None,
}

/// Possible types of cards
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum CardType {
    Spell,
    Weapon,
    Artifact,
    Minion,
    Project,
    Scheme,
    Upgrade,
    Identity,
}

/// Subtypes of cards
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum CardSubtype {
    Silvered,
}

/// Describes a boost ability activation
#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoostData {
    /// Boosted card
    pub card_id: CardId,
    /// How many times was the boost applied?
    pub count: u32,
}
