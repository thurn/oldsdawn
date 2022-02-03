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
use data::actions::PromptAction;
use data::card_state::CardPosition;
use data::delegates::{CastCardEvent, DawnEvent, DuskEvent, PayCardCostsEvent};
use data::game::GameState;
use data::primitives::{CardId, CardType, ItemLocation, RoomId, RoomLocation, Side};
use data::updates::GameUpdate;
use tracing::{info, instrument};

use crate::{dispatch, flags, mutations, queries, raid_actions};

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
    let card_id = mutations::realize_top_of_deck(game, user_side, 1)[0];
    mutations::set_revealed_to(game, card_id, user_side, true);
    mutations::spend_action_points(game, user_side, 1);
    mutations::move_card(game, card_id, CardPosition::Hand(user_side));
    check_end_turn(game, user_side);
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
        mutations::set_revealed_to(game, card_id, user_side.opponent(), true);
    }

    mutations::move_card(game, card_id, new_position);

    check_end_turn(game, user_side);
    Ok(())
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
    check_end_turn(game, user_side);
    Ok(())
}

/// Handles a [PromptAction] for the `user_side` player. Clears active prompts.
pub fn handle_prompt_action(
    game: &mut GameState,
    user_side: Side,
    action: PromptAction,
) -> Result<()> {
    ensure!(
        matches!(
            &game.player(user_side).prompt,
            Some(prompt) if prompt.responses.iter().any(|p| p == &action)
        ),
        "Unexpected action {:?} received",
        action
    );
    mutations::clear_prompts(game);

    match action {
        PromptAction::ActivateRoomAction(data) => {
            raid_actions::room_activation_action(game, user_side, data)
        }
        PromptAction::EncounterAction(data) => {
            raid_actions::encounter_action(game, user_side, data)
        }
        PromptAction::ContinueAction(data) => raid_actions::continue_action(game, user_side, data),
        PromptAction::RaidDestroyCard(card_id) => {
            raid_actions::destroy_card_action(game, user_side, card_id)
        }
        PromptAction::RaidScoreCard(card_id) => {
            raid_actions::score_card_action(game, user_side, card_id)
        }
        PromptAction::EndRaid => raid_actions::raid_end_action(game, user_side),
    }
}

/// Invoked after taking a game action to check if the turn should be
/// switched for the provided player.
pub fn check_end_turn(game: &mut GameState, side: Side) {
    if game.data.turn == side && game.player(side).actions == 0 {
        let new_turn = side.opponent();
        info!(?new_turn, "start_player_turn");
        game.data.turn = new_turn;
        if side == Side::Champion {
            game.data.turn_number += 1;
            dispatch::invoke_event(game, DuskEvent(game.data.turn_number));
        } else {
            dispatch::invoke_event(game, DawnEvent(game.data.turn_number));
        }
        game.player_mut(new_turn).actions = queries::start_of_turn_action_count(game, new_turn);
        game.updates.push(GameUpdate::StartTurn(new_turn));
    }
}
