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

//! Contains functions for responding to user-initiated game actions received from the client.
//!
//! By convention, functions in this module are responsible for validating the legality of
//! requests and returning [Result] accordingly. Beyond this point, game functions typically assume
//! the game is in a valid state and will panic if that is not true.

use crate::{mutations, queries};
use anyhow::{anyhow, Context, Result};
use data::card_state::CardPosition;
use data::game::GameState;
use data::primitives::Side;

/// The basic game action to draw a card.
pub fn draw_card(game: &mut GameState, side: Side) -> Result<()> {
    check_in_main_phase(game, side, |game, side| {
        let card = queries::top_of_deck(game, side).with_context(|| "Deck is empty!")?;
        mutations::spend_action_points(game, side, 1);
        mutations::move_card(game, card, CardPosition::Hand(side));
        Ok(())
    })
}

// Validates that the indicated player is currently in their Main phase, i.e. that it is their
// turn, that they have action points available, that a raid is not currently ongoing, that we are
// not currently waiting for an interface prompt response, etc.
fn check_in_main_phase(
    game: &mut GameState,
    side: Side,
    function: fn(&mut GameState, Side) -> Result<()>,
) -> Result<()> {
    if game.player(side).actions == 0 {
        Err(anyhow!("No action points available for {:?}", side))
    } else if game.data.turn != side {
        Err(anyhow!("Not currently {:?}'s turn", side))
    } else if game.data.raid.is_some() {
        Err(anyhow!("Raid is currently active"))
    } else {
        function(game, side)
    }
}
