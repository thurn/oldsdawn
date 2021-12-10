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
use crate::events::EventHandler;
use crate::primitives::{
    ActionCount, CardSubtype, CardType, HealthValue, ManaValue, Rarity, School, ShieldValue, Side,
    SpriteAddress,
};

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct CardTitle(pub String);

/// Rules text to display on a card
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct CardText {
    pub paragraphs: Vec<String>,
}

/// Cost to play a card
#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct CardCost {
    pub mana: ManaValue,
    pub actions: ActionCount,
}

/// Card numeric values
#[derive(PartialEq, Eq, Hash, Debug, Clone, Default)]
pub struct CardStats {
    pub health: Option<HealthValue>,
    pub attack: Option<HealthValue>,
    pub shield: Option<ShieldValue>,
}

/// Individual card configuration, properties which are not universal
#[derive(Debug, Default)]
pub struct CardConfig {
    pub handlers: Vec<EventHandler>,
    pub subtypes: Vec<CardSubtype>,
    pub stats: CardStats,
}

/// The fundamental object defining the behavior of a given card in Spelldawn
///
/// This struct contains properties which every card has
#[derive(Debug)]
pub struct CardDefinition {
    pub name: CardName,
    pub title: CardTitle,
    pub text: CardText,
    pub cost: CardCost,
    pub image: SpriteAddress,
    pub card_type: CardType,
    pub side: Side,
    pub school: School,
    pub rarity: Rarity,
    pub behavior: CardConfig,
}
