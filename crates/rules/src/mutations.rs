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

use std::cmp;

#[allow(unused)] // Used in rustdocs
use data::card_state::{CardData, CardPosition, CardPositionKind};
use data::delegates::{
    CardMoved, DawnEvent, DrawCardEvent, DuskEvent, MoveCardEvent, OverlordScoreCardEvent,
    PlayCardEvent, RaidEndEvent, Scope, StoredManaTakenEvent,
};
use data::game::{CurrentTurn, GameOverData, GamePhase, GameState, MulliganDecision};
use data::game_actions::{Prompt, PromptAction};
use data::primitives::{
    ActionCount, BoostData, CardId, ManaValue, PointsValue, RoomId, RoomLocation, Side, TurnNumber,
};
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
    info!(?card_id, ?new_position, "move_card");
    let old_position = game.card(card_id).position();
    game.move_card(card_id, new_position);

    dispatch::invoke_event(game, MoveCardEvent(CardMoved { old_position, new_position }));

    if old_position.in_deck() && new_position.in_hand() {
        dispatch::invoke_event(game, DrawCardEvent(card_id));
        game.updates.push(GameUpdate::DrawCard(card_id));
    }

    if !old_position.in_play() && new_position.in_play() {
        dispatch::invoke_event(game, PlayCardEvent(card_id));
    }

    if new_position.kind() == CardPositionKind::DeckUnknown {
        game.updates.push(GameUpdate::ShuffleIntoDeck(card_id));
    }

    if new_position.in_discard_pile()
        || new_position.kind() == CardPositionKind::Room
        || new_position.kind() == CardPositionKind::ArenaItem
    {
        game.updates.push(GameUpdate::MoveToZone(card_id));
    }
}

/// Helper to move all cards in a list to a new [CardPosition] via [move_card].
pub fn move_cards(game: &mut GameState, cards: &[CardId], to_position: CardPosition) {
    for card_id in cards {
        move_card(game, *card_id, to_position);
    }
}

// Shuffles the provided `cards` into the `side` player's deck, clearing their
// revealed state for both players.
pub fn shuffle_into_deck(game: &mut GameState, side: Side, cards: &[CardId]) {
    move_cards(game, cards, CardPosition::DeckUnknown(side));
    for card_id in cards {
        game.card_mut(*card_id).turn_face_down();
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
    move_cards(game, &cards, CardPosition::DeckUnknown(side));
}

/// Switches a card to be face-up and revealed to all players.
pub fn turn_face_up(game: &mut GameState, card_id: CardId) {
    let was_revealed_to_opponent = game.card(card_id).is_revealed_to(card_id.side.opponent());
    game.card_mut(card_id).turn_face_up();
    if !was_revealed_to_opponent {
        game.updates.push(GameUpdate::RevealToOpponent(card_id));
    }
}

/// Updates the 'revealed' state of a card to be visible to the indicated `side`
/// player. Note that this is *not* the same as [turn_face_up].
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
/// [GameUpdate] values will be appended for each draw. Returns a vector of the
/// newly-drawn [CardId]s.
pub fn draw_cards(game: &mut GameState, side: Side, count: usize) -> Vec<CardId> {
    let card_ids = realize_top_of_deck(game, side, count);
    for card_id in &card_ids {
        set_revealed_to(game, *card_id, side, true);
        move_card(game, *card_id, CardPosition::Hand(side))
    }
    card_ids
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

/// Adds points to a player's score and checks for the Game Over condition.
pub fn score_points(game: &mut GameState, side: Side, amount: PointsValue) {
    game.player_mut(side).score += amount;
    if game.player(side).score >= 7 {
        game.data.phase = GamePhase::GameOver(GameOverData { winner: side });
        game.updates.push(GameUpdate::GameOver(side));
    }
}

/// Takes *up to* `maximum` stored mana from a card and gives it to the player
/// who owns this card. If no mana remains, the card is moved to its owner's
/// discard pile.
#[instrument(skip(game))]
pub fn take_stored_mana(game: &mut GameState, card_id: CardId, maximum: ManaValue) {
    info!(?card_id, ?maximum, "take_stored_mana");
    let available = game.card(card_id).data.stored_mana;
    let taken = cmp::min(available, maximum);
    game.card_mut(card_id).data.stored_mana -= taken;
    gain_mana(game, card_id.side, taken);
    dispatch::invoke_event(game, StoredManaTakenEvent(card_id));

    if game.card(card_id).data.stored_mana == 0 {
        move_card(game, card_id, CardPosition::DiscardPile(card_id.side));
    }
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

/// Clears shown prompt a player.
pub fn clear_prompt(game: &mut GameState, side: Side) {
    game.player_mut(side).prompt = None;
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
    draw_cards(game, Side::Overlord, 5);
    set_prompt(game, Side::Overlord, prompt.clone());
    draw_cards(game, Side::Champion, 5);
    set_prompt(game, Side::Champion, prompt);
}

/// Invoked after a mulligan decision is received in order to check if the game
/// should be started.
///
/// Handles assigning initial mana & action points to players.
#[instrument(skip(game))]
pub fn check_start_game(game: &mut GameState) {
    match &game.data.phase {
        GamePhase::ResolveMulligans(mulligans)
            if mulligans.overlord.is_some() && mulligans.champion.is_some() =>
        {
            game.overlord.mana = 5;
            game.champion.mana = 5;
            start_turn(game, Side::Overlord, 1);
        }
        _ => {}
    }
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
pub fn realize_top_of_deck(game: &mut GameState, side: Side, count: usize) -> Vec<CardId> {
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
        move_card(game, *card_id, CardPosition::DeckTop(side));
    }

    card_ids
}

/// Invoked after taking a game action to check if the turn should be switched
/// for the provided player.
pub fn check_end_turn(game: &mut GameState, side: Side) {
    let turn = match &game.data.phase {
        GamePhase::Play(turn) => turn,
        _ => return,
    };

    if turn.side == side && game.player(side).actions == 0 {
        let turn_number = match side {
            Side::Overlord => turn.turn_number,
            Side::Champion => turn.turn_number + 1,
        };
        let next_side = side.opponent();
        start_turn(game, next_side, turn_number);
    }
}

/// Increases the level of all `can_level_up` Overlord cards in a room by 1. If
/// a card's level reaches its `level_requirement`, that card is immediately
/// scored and moved to the Overlord score zone.
///
/// Does not spend mana/actions etc.
pub fn level_up_room(game: &mut GameState, room_id: RoomId) {
    let mut scored = vec![];
    for occupant in game
        .cards_in_position_mut(Side::Overlord, CardPosition::Room(room_id, RoomLocation::Occupant))
        .filter(|card| crate::get(card.name).config.stats.can_level_up)
    {
        occupant.data.card_level += 1;

        if let Some(scheme_points) = crate::get(occupant.name).config.stats.scheme_points {
            if occupant.data.card_level >= scheme_points.level_requirement {
                scored.push((occupant.id, scheme_points));
            }
        }
    }

    for (card_id, scheme_points) in scored {
        turn_face_up(game, card_id);
        move_card(game, card_id, CardPosition::Scored(Side::Overlord));
        dispatch::invoke_event(game, OverlordScoreCardEvent(card_id));
        game.updates.push(GameUpdate::OverlordScoreCard(card_id, scheme_points.points));
        score_points(game, Side::Overlord, scheme_points.points);
    }
}

/// Attempt to pay a card's cost and turn it face up. Has no effect if the card
/// is not in play, already face up, or if the cost cannot be paid.
///
/// Returns true if the card was unveiled.
pub fn unveil_card(game: &mut GameState, card_id: CardId) -> bool {
    if game.card(card_id).is_face_down() && game.card(card_id).position().in_play() {
        match queries::mana_cost(game, card_id) {
            None => {
                turn_face_up(game, card_id);
                true
            }
            Some(cost) if cost <= game.player(card_id.side).mana => {
                spend_mana(game, card_id.side, cost);
                turn_face_up(game, card_id);
                true
            }
            _ => false,
        }
    } else {
        false
    }
}

/// Starts the turn for the `next_side` player.
fn start_turn(game: &mut GameState, next_side: Side, turn_number: TurnNumber) {
    game.data.phase = GamePhase::Play(CurrentTurn { side: next_side, turn_number });

    info!(?next_side, "start_player_turn");
    if next_side == Side::Overlord {
        dispatch::invoke_event(game, DuskEvent(turn_number));
    } else {
        dispatch::invoke_event(game, DawnEvent(turn_number));
    }
    game.player_mut(next_side).actions = queries::start_of_turn_action_count(game, next_side);
    game.updates.push(GameUpdate::StartTurn(next_side));

    draw_cards(game, next_side, 1);
}
