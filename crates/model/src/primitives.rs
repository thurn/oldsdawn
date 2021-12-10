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

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct CardId(pub usize);

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct EventId(pub u32);

pub type TurnNumber = u32;
pub type ManaValue = u32;
pub type ActionCount = u32;
pub type Score = u32;
pub type HealthValue = u32;
pub type ShieldValue = u32;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct SpriteAddress(pub String);

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum Side {
    Champion,
    Overlord,
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum School {
    Shadow,
    Nature,
    Time,
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
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

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum RoomLocation {
    Defender,
    InRoom,
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum ItemLocation {
    Weapons,
    NonWeapons,
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum Faction {
    Mortal,
    Abyssal,
    Infernal,
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Epic,
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum CardType {
    Spell,
    Weapon,
    Minion,
    Project,
    Scheme,
    Identity,
    Token,
}

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum CardSubtype {
    Silvered,
}
