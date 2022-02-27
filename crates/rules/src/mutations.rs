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

//! Core game mutations. In general, functions in this module append updates to
//! [GameState::updates]. Functions in this module panic if their preconditions
//! are not met, the higher-level game UI is responsible for ensuring this does
//! not happen.
//!
//! Generally, mutation functions are expected to invoke a delegate event
//! *after* performing their mutation to inform other systems that game state
//! has changed.

use data::actions::{Prompt, PromptAction};
#[allow(unused)] // Used in rustdocs
use data::card_state::{CardData, CardPosition, CardPositionKind};
use data::delegates::{
    CardMoved, DawnEvent, DrawCardEvent, DuskEvent, MoveCardEvent, PlayCardEvent, RaidEndEvent,
    Scope, StoredManaTakenEvent,
};
use data::game::{CurrentTurn, GamePhase, GameState, MulliganDecision};
use data::primitives::{ActionCount, BoostData, CardId, ManaValue, Side};
use data::updates::GameUpdate;
use rand::seq::IteratorRandom;
use tracing::{info, instrument};

use crate::{dispatch, queries};

/// Move a card to a new position. Detects cases like drawing cards, playing
/// cards, and shuffling cards back into the deck and fires events
/// appropriately. The card will be placed in the position in global sorting-key
/// order, via [GameState::move_card].
///
/// This function does *not* handle changing the 'revealed' state of the card,
/// the caller is responsible for updating that when the card moves to a new
/// game zone.
#[instrument(skip(game))]
pub fn move_card(game: &mut GameState, card_id: CardId, new_position: CardPosition) {
    move_card_internal(game, card_id, new_position, true)
}

/// Implementation for [move_card] which exposes the ability to turn off
/// updates.
fn move_card_internal(
    game: &mut GameState,
    card_id: CardId,
    new_position: CardPosition,
    mut push_update: bool,
) {
    info!(?card_id, ?new_position, "move_card");
    let old_position = game.card(card_id).position();
    game.move_card(card_id, new_position);

    dispatch::invoke_event(game, MoveCardEvent(CardMoved { old_position, new_position }));

    if push_update && old_position.in_deck() && new_position.in_hand() {
        dispatch::invoke_event(game, DrawCardEvent(card_id));
        game.updates.push(GameUpdate::DrawCard(card_id));
        push_update = false;
    }

    if !old_position.in_play() && new_position.in_play() {
        dispatch::invoke_event(game, PlayCardEvent(card_id));
    }

    if push_update && new_position.kind() == CardPositionKind::DeckUnknown {
        game.updates.push(GameUpdate::ShuffleIntoDeck(card_id));
        push_update = false;
    }

    if push_update {
        game.updates.push(GameUpdate::MoveCard(card_id));
    }
}

/// Helper to move all cards in a list to a new [CardPosition] via [move_card].
pub fn move_cards(
    game: &mut GameState,
    cards: &[CardId],
    to_position: CardPosition,
    push_updates: bool,
) {
    for card_id in cards {
        move_card_internal(game, *card_id, to_position, push_updates);
    }
}

// Shuffles the provided `cards` into the `side` player's deck, clearing their
// revealed state for both players.
pub fn shuffle_into_deck(game: &mut GameState, side: Side, cards: &[CardId], push_updates: bool) {
    move_cards(game, cards, CardPosition::DeckUnknown(side), push_updates);
    for card_id in cards {
        set_revealed_to(game, *card_id, Side::Overlord, false);
        set_revealed_to(game, *card_id, Side::Champion, false);
    }
    shuffle_deck(game, side);
}

/// Shuffles the `side` player's deck, moving all cards into the `DeckUnknown`
/// card position.
pub fn shuffle_deck(game: &mut GameState, side: Side) {
    let cards =
        game.cards_in_position(side, CardPosition::DeckTop(side)).map(|c| c.id).collect::<Vec<_>>();
    move_cards(game, &cards, CardPosition::DeckUnknown(side), false);
}

/// Updates the 'revealed' state of a card to be visible to the indicated `side`
/// player.
///
/// Appends [GameUpdate::RevealToOpponent] if the new state is revealed to the
/// opponent.
#[instrument(skip(game))]
pub fn set_revealed_to(game: &mut GameState, card_id: CardId, side: Side, revealed: bool) {
    let current = game.card(card_id).is_revealed_to(side);
    game.card_mut(card_id).set_revealed_to(side, revealed);

    if side != card_id.side && !current && revealed {
        game.updates.push(GameUpdate::RevealToOpponent(card_id));
    }
}

/// Helper function to draw `count` cards from the top of a player's deck and
/// place them into their hand.
///
/// Cards are set as revealed to the `side` player. If `push_updates` is true,
/// [GameUpdate] values will be appended for each draw.
pub fn draw_cards(game: &mut GameState, side: Side, count: usize, push_updates: bool) {
    let card_ids = realize_top_of_deck(game, side, count, push_updates);
    for card_id in card_ids {
        set_revealed_to(game, card_id, side, true);
        move_card_internal(game, card_id, CardPosition::Hand(side), push_updates)
    }
}

/// Give mana to the indicated player.
#[instrument(skip(game))]
pub fn gain_mana(game: &mut GameState, side: Side, amount: ManaValue) {
    info!(?side, ?amount, "gain_mana");
    game.player_mut(side).mana += amount;
}

/// Spends a player's mana. Panics if sufficient mana is not available
/// [instrument(skip(game))]
pub fn spend_mana(game: &mut GameState, side: Side, amount: ManaValue) {
    info!(?side, ?amount, "spend_mana");
    assert!(game.player(side).mana >= amount, "Insufficient mana available");
    game.player_mut(side).mana -= amount;
}

/// Spends a player's action points.
///
/// Panics if sufficient action points are not available.
#[instrument(skip(game))]
pub fn spend_action_points(game: &mut GameState, side: Side, amount: ActionCount) {
    info!(?side, ?amount, "spend_action_points");
    assert!(game.player(side).actions >= amount, "Insufficient action points available");
    game.player_mut(side).actions -= amount;
}

/// Takes *up to* `maximum` stored mana from a card and gives it to the player
/// who owns this card.
#[instrument(skip(game))]
pub fn take_stored_mana(game: &mut GameState, card_id: CardId, maximum: ManaValue) {
    info!(?card_id, ?maximum, "take_stored_mana");
    let available = game.card(card_id).data.stored_mana;
    let taken = std::cmp::min(available, maximum);
    game.card_mut(card_id).data.stored_mana -= taken;
    gain_mana(game, card_id.side, taken);
    dispatch::invoke_event(game, StoredManaTakenEvent(card_id));
}

/// Overwrites the value of [CardData::boost_count] to match the provided
/// [BoostData].
#[instrument(skip(game))]
pub fn write_boost(game: &mut GameState, scope: Scope, data: BoostData) {
    info!(?scope, ?data, "write_boost");
    game.card_mut(data.card_id).data.boost_count = data.count;
}

/// Set the boost count to zero for the card in `scope`.
#[instrument(skip(game))]
pub fn clear_boost<T>(game: &mut GameState, scope: Scope, _: T) {
    info!(?scope, "clear_boost");
    game.card_mut(scope.card_id()).data.boost_count = 0;
}

/// Sets the current prompt for the `side` player to the provided
/// [Prompt]. Panics if a prompt is already set for this player.
pub fn set_prompt(game: &mut GameState, side: Side, prompt: Prompt) {
    assert!(game.player(side).prompt.is_none(), "Player {:?} already has an active prompt", side);
    game.player_mut(side).prompt = Some(prompt);
}

/// Clears shown prompts for both players.
pub fn clear_prompts(game: &mut GameState) {
    game.overlord.prompt = None;
    game.champion.prompt = None;
}

/// Ends the current raid. Panics if no raid is currently active.
#[instrument(skip(game))]
pub fn end_raid(game: &mut GameState) {
    info!("end_raid");
    let raid = game.raid().expect("Active raid").raid_id;
    game.data.raid = None;
    dispatch::invoke_event(game, RaidEndEvent(raid));
    check_end_turn(game, Side::Champion)
}

/// Deals initial hands to both players and prompts for mulligan decisions.
#[instrument(skip(game))]
pub fn deal_opening_hands(game: &mut GameState) {
    info!("deal_opening_hands");
    let prompt = Prompt {
        context: None,
        responses: vec![
            PromptAction::MulliganDecision(MulliganDecision::Keep),
            PromptAction::MulliganDecision(MulliganDecision::Mulligan),
        ],
    };
    draw_cards(game, Side::Overlord, 5, false);
    set_prompt(game, Side::Overlord, prompt.clone());
    draw_cards(game, Side::Champion, 5, false);
    set_prompt(game, Side::Champion, prompt);
}

/// Returns a list of `count` cards from the top of the `side` player's
/// deck, in sorting-key order (later indices are are closer to the top
/// of the deck).
///
/// Selects randomly unless cards are already known to be in this position.
/// If insufficient cards are present in the deck, returns all available
/// cards. Cards are moved to their new positions via [move_card], meaning that
/// subsequent calls to this function will see the same results.
///
/// Does not change the 'revealed' state of cards.
pub fn realize_top_of_deck(
    game: &mut GameState,
    side: Side,
    count: usize,
    push_updates: bool,
) -> Vec<CardId> {
    let mut cards = game.card_list_for_position(side, CardPosition::DeckTop(side));
    let result = if count <= cards.len() {
        cards[0..count].to_vec()
    } else {
        let remaining = count - cards.len();
        let unknown = game.cards_in_position(side, CardPosition::DeckUnknown(side));
        let mut shuffled = if game.data.config.deterministic {
            unknown.take(remaining).collect()
        } else {
            unknown.choose_multiple(&mut rand::thread_rng(), remaining)
        };
        shuffled.append(&mut cards);
        shuffled
    };
    let card_ids = result.into_iter().map(|c| c.id).collect::<Vec<_>>();
    assert_eq!(card_ids.len(), count);

    for card_id in &card_ids {
        move_card_internal(game, *card_id, CardPosition::DeckTop(side), push_updates);
    }

    card_ids
}

/// Invoked after taking a game action to check if the turn should be switched
/// for the provided player.
///
/// Panics if the provided game is not currently active.
pub fn check_end_turn(game: &mut GameState, side: Side) {
    let turn = game.current_turn().expect("current_turn");

    if turn.side == side && game.player(side).actions == 0 {
        let turn_number = match side {
            Side::Overlord => turn.turn_number,
            Side::Champion => turn.turn_number + 1,
        };
        let next_side = side.opponent();
        game.data.phase = GamePhase::Play(CurrentTurn { side: next_side, turn_number });

        info!(?next_side, "start_player_turn");
        if side == Side::Champion {
            dispatch::invoke_event(game, DuskEvent(turn_number));
        } else {
            dispatch::invoke_event(game, DawnEvent(turn_number));
        }
        game.player_mut(next_side).actions = queries::start_of_turn_action_count(game, next_side);
        game.updates.push(GameUpdate::StartTurn(next_side));
    }
}
