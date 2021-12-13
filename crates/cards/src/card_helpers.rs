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

//! Helpers for defining card behaviors. This file is intended be be used via wildcard import in
//! card definition files.

use model::card_definition::{Ability, AbilityType, CardStats, CardText, Cost, PowerUp};
use model::delegates::{Context, Delegate, EventDelegate, MutationFn};
use model::game::GameState;
use model::primitives::{AbilityId, AttackValue, CardId, ManaValue, SpriteAddress};

/// Provides the rules text for a card
pub fn text(text: &str) -> CardText {
    CardText { text: text.to_owned() }
}

/// Provides the cost for a card
pub fn cost(mana: ManaValue) -> Cost {
    Cost { mana, actions: 1 }
}

/// Provides an image for a card
pub fn sprite(text: &str) -> SpriteAddress {
    SpriteAddress(text.to_owned())
}

/// A RequirementFn which restricts delegates to only listen to events for their own ability.
pub fn this_ability(game: &GameState, context: Context, ability_id: AbilityId) -> bool {
    context.ability_id() == ability_id
}

/// A RequirementFn which restricts delegates to only listen to events for their own card.
pub fn this_card(game: &GameState, context: Context, card_id: CardId) -> bool {
    context.card_id() == card_id
}

/// An ability which triggers when a card is played
pub fn on_play(rules: &str, mutation: MutationFn<CardId>) -> Ability {
    Ability {
        text: text(rules),
        ability_type: AbilityType::Standard,
        delegates: vec![Delegate::OnPlayCard(EventDelegate { requirement: this_card, mutation })],
    }
}

/// Give mana to the player who owns this delegate
pub fn gain_mana(game: &mut GameState, context: Context, amount: ManaValue) {
    game.player_state_mut(context.side()).mana += amount;
}

pub struct WeaponStats {
    pub base_attack: AttackValue,
    pub bonus_cost: ManaValue,
    pub attack_bonus: AttackValue,
}

impl From<WeaponStats> for CardStats {
    fn from(stats: WeaponStats) -> Self {
        Self {
            base_attack: Some(stats.base_attack),
            power_up: Some(PowerUp { cost: stats.bonus_cost, bonus: stats.attack_bonus }),
            ..Self::default()
        }
    }
}
