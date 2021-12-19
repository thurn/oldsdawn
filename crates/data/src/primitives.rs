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
use serde::{Deserialize, Serialize};

pub type TurnNumber = u32;
pub type ManaValue = u32;
pub type ActionCount = u32;
pub type PointsValue = u32;
pub type HealthValue = u32;
pub type AttackValue = u32;
pub type ShieldValue = u32;
pub type BoostCount = u32;
pub type LevelValue = u32;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct GameId {
    pub value: u64,
}

impl GameId {
    pub fn new(value: u64) -> Self {
        Self { value }
    }

    pub fn key(&self) -> [u8; 8] {
        self.value.to_be_bytes()
    }
}

/// The two players in a game: Overlord & Champion
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Side {
    Overlord,
    Champion,
}

impl Side {
    pub fn opponent(&self) -> Self {
        match self {
            Side::Champion => Side::Overlord,
            Side::Overlord => Side::Champion,
        }
    }
}

/// Identifies a card in an ongoing game
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct CardId {
    pub side: Side,
    pub index: usize,
}

impl CardId {
    pub fn new(side: Side, index: usize) -> Self {
        Self { side, index }
    }
}

/// Identifies an ability within a card. Abilities are the only game entity which may contain
/// delegates. Abilities are identified by their position within the card's 'abilities',
/// or 'activated_abilities' vector.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Ord, PartialOrd, Serialize, Deserialize)]
pub struct AbilityIndex(pub usize);

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct AbilityId {
    pub card_id: CardId,
    pub index: AbilityIndex,
}

impl AbilityId {
    pub fn new(card_id: CardId, index: usize) -> Self {
        Self { card_id, index: AbilityIndex(index) }
    }
}

impl From<AbilityId> for CardId {
    fn from(id: AbilityId) -> Self {
        id.card_id
    }
}

/// Uniquely identifies a raid within a given game
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct RaidId(pub u32);

#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub struct Sprite {
    pub address: String,
}

impl Sprite {
    pub fn new(address: impl Into<String>) -> Self {
        Self { address: address.into() }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum School {
    Neutral,
    Shadow,
    Nature,
    Time,
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum RoomId {
    Treasury,
    Sanctum,
    Crypts,
    RoomA,
    RoomB,
    RoomC,
    RoomD,
    RoomE,
}

pub type DefenderIndex = u32;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum RoomLocation {
    Defender,
    InRoom,
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ItemLocation {
    Weapons,
    Artifacts,
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Faction {
    Mortal,
    Abyssal,
    Infernal,
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,

    /// Card cannot be obtained via random rewards
    None,
}

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
    Token,
}

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

impl From<BoostData> for CardId {
    fn from(data: BoostData) -> Self {
        data.card_id
    }
}
