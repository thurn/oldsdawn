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

//! Defines handling for the basic top-level game actions a player can take.

use anyhow::{anyhow, ensure, Context, Result};
use data::card_state::CardPosition;
use data::delegates::{CastCardEvent, DawnEvent, DuskEvent, PayCardCostsEvent};
use data::game::GameState;
use data::primitives::{CardId, CardType, ItemLocation, RoomId, RoomLocation, Side};
use data::prompt::PromptResponse;
use data::updates::GameUpdate;
use tracing::{info, instrument};

use crate::{dispatch, flags, mutations, queries, raid};

/// The basic game action to draw a card during your turn by spending one
/// action.
#[instrument(skip(game))]
pub fn draw_card_action(game: &mut GameState, user_side: Side) -> Result<()> {
    info!(?user_side, "draw_card_action");
    ensure!(
        flags::can_take_draw_card_action(game, user_side),
        "Cannot draw card for {:?}",
        user_side
    );
    let card = queries::top_of_deck(game, user_side).with_context(|| "Deck is empty!")?;
    mutations::spend_action_points(game, user_side, 1);
    mutations::move_card(game, card, CardPosition::Hand(user_side));
    check_end_turn(game, user_side)
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
    /// Gets the RoomId targeted by a player, or returns an error if no target
    /// was provided.
    pub fn room_id(&self) -> Result<RoomId> {
        match self {
            PlayCardTarget::Room(room_id) => Ok(*room_id),
            _ => Err(anyhow!("Expected a RoomId to be provided but got {:?}", self)),
        }
    }
}

/// The basic game action to play a card during your turn. Spends the resource
/// cost for a card, resolves its effects, and then moves it to the appropriate
/// new [CardPosition]. Spell, Weapon, and Artifact cards are immediately
/// revealed when played.
#[instrument(skip(game))]
pub fn play_card_action(
    game: &mut GameState,
    user_side: Side,
    card_id: CardId,
    target: PlayCardTarget,
) -> Result<()> {
    info!(?user_side, ?card_id, ?target, "play_card_action");
    ensure!(
        flags::can_take_play_card_action(game, user_side, card_id),
        "Cannot play card {:?}",
        card_id
    );
    let card = game.card(card_id);
    let definition = crate::get(card.name);
    let enters_revealed = flags::enters_play_revealed(game, card_id);

    if enters_revealed {
        mutations::spend_mana(
            game,
            user_side,
            queries::mana_cost(game, card_id).with_context(|| "Card has no mana cost")?,
        );
    }

    mutations::spend_action_points(game, user_side, definition.cost.actions);
    dispatch::invoke_event(game, PayCardCostsEvent(card_id));

    dispatch::invoke_event(game, CastCardEvent(card_id));

    let new_position = match definition.card_type {
        CardType::Spell | CardType::Sorcery => CardPosition::DiscardPile(user_side),
        CardType::Weapon => CardPosition::ArenaItem(ItemLocation::Weapons),
        CardType::Artifact => CardPosition::ArenaItem(ItemLocation::Artifacts),
        CardType::Minion => CardPosition::Room(target.room_id()?, RoomLocation::Defender),
        CardType::Project | CardType::Scheme | CardType::Upgrade => {
            CardPosition::Room(target.room_id()?, RoomLocation::Occupant)
        }
        CardType::Identity => CardPosition::Identity(user_side),
    };

    if enters_revealed {
        mutations::set_revealed(game, card_id, true);
    }

    mutations::move_card(game, card_id, new_position);

    check_end_turn(game, user_side)
}

/// The basic game action to gain 1 mana during your turn by spending one
/// action.
#[instrument(skip(game))]
pub fn gain_mana_action(game: &mut GameState, user_side: Side) -> Result<()> {
    info!(?user_side, "gain_mana_action");
    ensure!(
        flags::can_take_gain_mana_action(game, user_side),
        "Cannot gain mana for {:?}",
        user_side
    );
    mutations::spend_action_points(game, user_side, 1);
    mutations::gain_mana(game, user_side, 1);
    check_end_turn(game, user_side)
}

/// Handles a [PromptResponse] in response to a prompt for the `user_side`
/// player. Clears active prompts.
pub fn handle_prompt_response(
    game: &mut GameState,
    user_side: Side,
    action: PromptResponse,
) -> Result<()> {
    ensure!(
        matches!(
            &game.player(user_side).prompt,
            Some(prompt) if prompt.responses.iter().any(|p| p.kind() == action.kind())
        ),
        "Unexpected action {:?} received",
        action
    );
    mutations::clear_prompts(game);

    match action {
        PromptResponse::ActivateRoomAction(data) => {
            raid::activate_room_action(game, user_side, data)
        }
        PromptResponse::EncounterAction(data) => raid::encounter_action(game, user_side, data),
        PromptResponse::AdvanceAction(data) => raid::advance_action(game, user_side, data),
        PromptResponse::RaidDestroyCard(card_id) => {
            raid::destroy_card_action(game, user_side, card_id)
        }
        PromptResponse::RaidScoreCard(card_id) => raid::score_card_action(game, user_side, card_id),
        PromptResponse::RaidEnd => raid::raid_end_action(game, user_side),
    }
}

/// Invoked after taking a primary game action to check if the turn should be
/// switched.
fn check_end_turn(game: &mut GameState, user_side: Side) -> Result<()> {
    ensure!(game.data.turn == user_side, "Not currently {:?}'s turn", user_side);
    if game.player(user_side).actions == 0 {
        let new_turn = user_side.opponent();
        info!(?new_turn, "start_player_turn");
        game.data.turn = new_turn;
        if user_side == Side::Champion {
            game.data.turn_number += 1;
            dispatch::invoke_event(game, DuskEvent(game.data.turn_number));
        } else {
            dispatch::invoke_event(game, DawnEvent(game.data.turn_number));
        }
        game.player_mut(new_turn).actions = queries::start_of_turn_action_count(game, new_turn);
        game.updates.push(GameUpdate::StartTurn(new_turn));
    }
    Ok(())
}
