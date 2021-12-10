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

use enumset::enum_set;
use model::card_definition::{CardCost, CardStats, CardText, CardTitle};
use model::events::{CardPositionType, EventCallback, EventContext, EventFn, EventHandler};
use model::game::GameState;
use model::primitives::{ManaValue, SpriteAddress};

pub fn event(callback: EventCallback) -> EventHandler {
    EventHandler { callback, active_positions: enum_set!(CardPositionType::Arena) }
}

pub fn title(text: &str) -> CardTitle {
    CardTitle(text.to_owned())
}

pub fn text(text: &str) -> CardText {
    CardText { paragraphs: vec![text.to_owned()] }
}

pub fn cost(amount: ManaValue) -> CardCost {
    CardCost { mana: amount, actions: 1 }
}

pub fn sprite(text: &str) -> SpriteAddress {
    SpriteAddress(text.to_owned())
}

pub fn on_play(callback: EventFn) -> Vec<EventHandler> {
    vec![event(EventCallback::OnPlay(callback))]
}

pub fn gain_mana(game: &mut GameState, context: EventContext, amount: ManaValue) {
    game.player_state_mut(context.side).mana += amount
}

pub fn attack(attack: u32) -> CardStats {
    CardStats { attack: Some(attack), ..CardStats::default() }
}
