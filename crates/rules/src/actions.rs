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

//! Contains functions for responding to user-initiated game actions received
//! from the client.
//!
//! By convention, functions in this module are responsible for validating the
//! legality of requests and returning [Result] accordingly. Beyond this point,
//! game functions typically assume the game is in a valid state and will panic
//! if that is not true.

use anyhow::{anyhow, bail, ensure, Context, Result};
use data::card_state::CardPosition;
use data::delegates::{self, CastCardEvent, PayCardCostsEvent};
use data::game::GameState;
use data::primitives::{CardId, CardType, ItemLocation, RoomId, RoomLocation, Side};
use tracing::info_span;

use crate::{dispatch, mutations, queries};

/// The basic game action to draw a card.
pub fn draw_card(game: &mut GameState, side: Side) -> Result<()> {
    ensure!(queries::in_main_phase(game, side), "Not in main phase for {:?}", side);
    let card = queries::top_of_deck(game, side).with_context(|| "Deck is empty!")?;
    mutations::spend_action_points(game, side, 1);
    mutations::move_card(game, card, CardPosition::Hand(side));
    Ok(())
}

/// Possible targets for the 'play card' action. Note that many types of targets
/// are *not* selected in the original PlayCard action request but are instead
/// selected via a follow-up prompt, and thus are not represented here.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum PlayCardTarget {
    None,
    Room(RoomId),
}

impl PlayCardTarget {
    pub fn room_id(&self) -> Result<RoomId> {
        match self {
            PlayCardTarget::Room(room_id) => Ok(*room_id),
            _ => Err(anyhow!("Expected a RoomId to be provided but got {:?}", self)),
        }
    }
}

/// The basic game action to play a card
pub fn play_card(
    game: &mut GameState,
    side: Side,
    card_id: CardId,
    target: PlayCardTarget,
) -> Result<()> {
    info_span!("play_card");
    ensure!(queries::can_play(game, side, card_id), "Cannot play card {:?}", card_id);
    let card = game.card(card_id);
    let definition = crate::get(card.name);

    if let Some(mana_cost) = definition.cost.mana {
        mutations::spend_mana(game, side, mana_cost);
    }
    mutations::spend_action_points(game, side, definition.cost.actions);
    dispatch::invoke_event(game, PayCardCostsEvent(card_id));

    dispatch::invoke_event(game, CastCardEvent(card_id));

    let new_position = match definition.card_type {
        CardType::Spell => CardPosition::DiscardPile(side),
        CardType::Weapon => CardPosition::ArenaItem(ItemLocation::Weapons),
        CardType::Artifact => CardPosition::ArenaItem(ItemLocation::Artifacts),
        CardType::Minion => CardPosition::Room(target.room_id()?, RoomLocation::Defender),
        CardType::Project | CardType::Scheme | CardType::Upgrade => {
            CardPosition::Room(target.room_id()?, RoomLocation::InRoom)
        }
        CardType::Identity => CardPosition::Identity(side),
    };
    mutations::move_card(game, card_id, new_position);
    Ok(())
}
