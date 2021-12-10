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

use crate::game::GameState;
use crate::primitives::{CardId, EventId, Side};

use enumset::{EnumSet, EnumSetType};

use std::fmt;
use std::fmt::{Debug, Formatter};

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct EventContext {
    pub event_id: EventId,
    pub side: Side,
    pub this: CardId,
}

// pub type EventFn = fn(game: &mut GameState, context: EventContext);
// pub type DataEventFn<T> = fn(game: &mut GameState, context: EventContext, data: T);

pub type EventFn = fn(&mut GameState, EventContext);
pub type DataEventFn<T> = fn(&mut GameState, EventContext, T);

/// Represents an event which occurs during a game of Spelldawn, with any associated arguments.
///
/// Event names prefixed with "On" are lifecycle events, sent only to card which is the subject
/// of the event. All other events are global, sent to every eligible card.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum GameEvent {
    OverlordTurnBegins,
    OnPlay,
    PlayCard(CardId),
    DrawCard(CardId),
}

pub enum EventCallback {
    OverlordTurnBegins(EventFn),
    OnPlay(EventFn),
    PlayCard(DataEventFn<CardId>),
    DrawCard(DataEventFn<CardId>),
}

pub fn invoke_if_matching(
    game: &mut GameState,
    context: EventContext,
    event: GameEvent,
    handler: &EventCallback,
) {
    match (event, handler) {
        (GameEvent::OverlordTurnBegins, EventCallback::OverlordTurnBegins(handler)) => {
            handler(game, context)
        }
        (GameEvent::OnPlay, EventCallback::OnPlay(handler)) => handler(game, context),
        (GameEvent::PlayCard(card_id), EventCallback::PlayCard(handler))
        | (GameEvent::DrawCard(card_id), EventCallback::DrawCard(handler)) => {
            handler(game, context, card_id)
        }
        _ => {}
    }
}

impl Debug for EventCallback {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "<EventHandler>")
    }
}

#[derive(Debug, EnumSetType)]
pub enum CardPositionType {
    Arena,
    Hand,
    Scored,
    Discard,
    Deck,
}

#[derive(Debug)]
pub struct EventHandler {
    pub callback: EventCallback,

    /// Set of card positions in which this handler should be invoked -- typically events only
    /// fire when cards are in the arena, but certain cases require responding to events in other
    /// locations.
    pub active_positions: EnumSet<CardPositionType>,
}
