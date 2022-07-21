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
//! [GameState::updates].
//!
//! Generally, mutation functions are expected to invoke a delegate event
//! *after* performing their mutation to inform other systems that game state
//! has changed.

use std::cmp;

use anyhow::Result;
#[allow(unused)] // Used in rustdocs
use data::card_state::{CardData, CardPosition, CardPositionKind};
use data::delegates::{
    CardMoved, DawnEvent, DealtDamage, DealtDamageEvent, DrawCardEvent, DuskEvent, EnterPlayEvent,
    MoveCardEvent, OverlordScoreCardEvent, RaidEndEvent, RaidEnded, RaidFailureEvent, RaidOutcome,
    RaidSuccessEvent, Scope, ScoreCard, ScoreCardEvent, StoredManaTakenEvent, SummonMinionEvent,
    UnveilProjectEvent,
};
use data::game::{GameOverData, GamePhase, GameState, TurnData};
use data::game_actions::{CardPromptAction, GamePrompt};
use data::primitives::{
    ActionCount, BoostData, CardId, HasAbilityId, ManaValue, PointsValue, RoomId, RoomLocation,
    Side, TurnNumber,
};
use data::random;
use data::updates::GameUpdate;
use tracing::{info, instrument};
use with_error::verify;

use crate::mana::ManaPurpose;
use crate::{constants, dispatch, flags, mana, queries};

/// Move a card to a new position. Detects cases like drawing cards, playing
/// cards, and shuffling cards back into the deck and fires events
/// appropriately. The card will be placed in the position in global sorting-key
/// order, via [GameState::move_card_internal].
///
/// This function does *not* handle changing the 'revealed' or 'face down' state
/// of the card, the caller is responsible for updating that when the card moves
/// to a new game zone.
#[instrument(skip(game))]
pub fn move_card(game: &mut GameState, card_id: CardId, new_position: CardPosition) -> Result<()> {
    info!(?card_id, ?new_position, "move_card");
    let old_position = game.card(card_id).position();
    game.move_card_internal(card_id, new_position);

    dispatch::invoke_event(game, MoveCardEvent(CardMoved { old_position, new_position }))?;

    if old_position.in_deck() && new_position.in_hand() {
        dispatch::invoke_event(game, DrawCardEvent(card_id))?;
    }

    if !old_position.in_play() && new_position.in_play() {
        game.card_mut(card_id).data.last_entered_play = Some(game.data.turn);
        dispatch::invoke_event(game, EnterPlayEvent(card_id))?;
    }

    if !new_position.in_play() {
        clear_counters(game, card_id);
    }

    Ok(())
}

/// Helper to move all cards in a list to a new [CardPosition] via [move_card].
pub fn move_cards(game: &mut GameState, cards: &[CardId], to_position: CardPosition) -> Result<()> {
    for card_id in cards {
        move_card(game, *card_id, to_position)?;
    }
    Ok(())
}

/// Move a card to the discard pile. This should specifically be used when a
/// player's *own* effect causes their card to be discarded.
pub fn sacrifice_card(game: &mut GameState, card_id: CardId) -> Result<()> {
    move_card(game, card_id, CardPosition::DiscardPile(card_id.side))
}

// Shuffles the provided `cards` into the `side` player's deck, clearing their
// revealed state for both players.
pub fn shuffle_into_deck(game: &mut GameState, side: Side, cards: &[CardId]) -> Result<()> {
    move_cards(game, cards, CardPosition::DeckUnknown(side))?;
    for card_id in cards {
        game.card_mut(*card_id).turn_face_down();
        game.card_mut(*card_id).set_revealed_to(Side::Overlord, false);
        game.card_mut(*card_id).set_revealed_to(Side::Champion, false);
    }
    shuffle_deck(game, side)?;
    game.record_update(|| GameUpdate::ShuffleIntoDeck);
    Ok(())
}

/// Shuffles the `side` player's deck, moving all cards into the `DeckUnknown`
/// card position.
pub fn shuffle_deck(game: &mut GameState, side: Side) -> Result<()> {
    let cards =
        game.cards_in_position(side, CardPosition::DeckTop(side)).map(|c| c.id).collect::<Vec<_>>();
    move_cards(game, &cards, CardPosition::DeckUnknown(side))
}

/// Helper function to draw `count` cards from the top of a player's deck and
/// place them into their hand. If there are insufficient cards available, the
/// `side` player loses the game.
///
/// Cards are set as revealed to the `side` player. Returns a vector of the
/// newly-drawn [CardId]s.
pub fn draw_cards(game: &mut GameState, side: Side, count: u32) -> Result<Vec<CardId>> {
    let card_ids = realize_top_of_deck(game, side, count)?;

    if card_ids.len() != count as usize {
        game_over(game, side.opponent())?;
        return Ok(vec![]);
    }

    for card_id in &card_ids {
        game.card_mut(*card_id).set_revealed_to(side, true);
    }

    game.record_update(|| GameUpdate::DrawCards(side, card_ids.clone()));

    for card_id in &card_ids {
        move_card(game, *card_id, CardPosition::Hand(side))?;
    }

    Ok(card_ids)
}

/// Lose action points if a player has more than 0.
#[instrument(skip(game))]
pub fn lose_action_points_if_able(
    game: &mut GameState,
    side: Side,
    amount: ActionCount,
) -> Result<()> {
    if game.player(side).actions > 0 {
        spend_action_points(game, side, amount)?;
    }
    Ok(())
}

/// Spends a player's action points.
///
/// Returns an error if sufficient action points are not available.
#[instrument(skip(game))]
pub fn spend_action_points(game: &mut GameState, side: Side, amount: ActionCount) -> Result<()> {
    info!(?side, ?amount, "spend_action_points");
    verify!(game.player(side).actions >= amount, "Insufficient action points available");
    game.player_mut(side).actions -= amount;
    Ok(())
}

/// Adds points to a player's score and checks for the Game Over condition.
pub fn score_points(game: &mut GameState, side: Side, amount: PointsValue) -> Result<()> {
    game.player_mut(side).score += amount;
    if game.player(side).score >= 7 {
        game_over(game, side)?;
    }
    Ok(())
}

/// Mark the game as won by the `winner` player.
pub fn game_over(game: &mut GameState, winner: Side) -> Result<()> {
    game.data.phase = GamePhase::GameOver(GameOverData { winner });
    game.record_update(|| GameUpdate::GameOver(winner));
    Ok(())
}

/// Behavior when a card has no stored mana remaining after [take_stored_mana].
#[derive(Debug, Eq, PartialEq)]
pub enum OnZeroStored {
    Sacrifice,
    Ignore,
}

/// Takes *up to* `maximum` stored mana from a card and gives it to the player
/// who owns this card. Returns the amount of mana taken.
///
/// If no mana remains, the card is moved to its owner's discard pile if
/// `OnEmpty::MoveToDiscard` is specified.
#[instrument(skip(game))]
pub fn take_stored_mana(
    game: &mut GameState,
    card_id: CardId,
    maximum: ManaValue,
    on_zero_stored: OnZeroStored,
) -> Result<ManaValue> {
    info!(?card_id, ?maximum, "take_stored_mana");
    let available = game.card(card_id).data.stored_mana;
    let taken = cmp::min(available, maximum);
    game.card_mut(card_id).data.stored_mana -= taken;
    mana::gain(game, card_id.side, taken);
    dispatch::invoke_event(game, StoredManaTakenEvent(card_id))?;

    if on_zero_stored == OnZeroStored::Sacrifice && game.card(card_id).data.stored_mana == 0 {
        sacrifice_card(game, card_id)?;
    }

    Ok(taken)
}

/// Overwrites the value of [CardData::boost_count] to match the provided
/// [BoostData].
#[instrument(skip(game))]
pub fn write_boost(game: &mut GameState, scope: Scope, data: &BoostData) -> Result<()> {
    info!(?scope, ?data, "write_boost");
    game.card_mut(data.card_id).data.boost_count = data.count;
    Ok(())
}

/// Set the boost count to zero for the card in `scope`.
#[instrument(skip(game))]
pub fn clear_boost<T>(game: &mut GameState, scope: Scope, _: &T) -> Result<()> {
    info!(?scope, "clear_boost");
    game.card_mut(scope.card_id()).data.boost_count = 0;
    Ok(())
}

/// Sets the current prompt for the `side` player to contain the non-`None`
/// card actions in `actions`.
///
/// Returns an error if a prompt is already set for this player.
pub fn set_prompt(
    game: &mut GameState,
    side: Side,
    actions: Vec<Option<CardPromptAction>>,
) -> Result<()> {
    verify!(game.player(side).prompt.is_none(), "Prompt already present");
    game.player_mut(side).prompt =
        Some(GamePrompt::card_actions(actions.into_iter().flatten().collect()));
    Ok(())
}

/// Ends the current raid. Returns an error if no raid is currently active.
#[instrument(skip(game))]
pub fn end_raid(game: &mut GameState, outcome: RaidOutcome) -> Result<()> {
    info!("end_raid");
    let raid_id = game.raid()?.raid_id;
    match outcome {
        RaidOutcome::Success => dispatch::invoke_event(game, RaidSuccessEvent(raid_id))?,
        RaidOutcome::Failure => dispatch::invoke_event(game, RaidFailureEvent(raid_id))?,
    }
    dispatch::invoke_event(game, RaidEndEvent(RaidEnded { raid_id, outcome }))?;
    game.data.raid = None;
    check_end_turn(game)?;
    Ok(())
}

/// Deals initial hands to both players and prompts for mulligan decisions.
#[instrument(skip(game))]
pub fn deal_opening_hands(game: &mut GameState) -> Result<()> {
    info!("deal_opening_hands");
    draw_cards(game, Side::Overlord, constants::STARTING_HAND_SIZE)?;
    draw_cards(game, Side::Champion, constants::STARTING_HAND_SIZE)?;
    Ok(())
}

/// Invoked after a mulligan decision is received in order to check if the game
/// should be started.
///
/// Handles assigning initial mana & action points to players.
#[instrument(skip(game))]
pub fn check_start_game(game: &mut GameState) -> Result<()> {
    match &game.data.phase {
        GamePhase::ResolveMulligans(mulligans)
            if mulligans.overlord.is_some() && mulligans.champion.is_some() =>
        {
            mana::set(game, Side::Overlord, 5);
            mana::set(game, Side::Champion, 5);
            start_turn(game, Side::Overlord, 1)?;
        }
        _ => {}
    }
    Ok(())
}

/// Returns a list of *up to* `count` cards from the top of the `side` player's
/// deck, in sorting-key order (later indices are are closer to the top
/// of the deck).
///
/// Selects randomly unless cards are already known to be in this position.
/// If insufficient cards are present in the deck, returns all available
/// cards. Cards are moved to their new positions via [move_card], meaning that
/// subsequent calls to this function will see the same results.
///
/// Does not change the 'revealed' state of cards.
pub fn realize_top_of_deck(game: &mut GameState, side: Side, count: u32) -> Result<Vec<CardId>> {
    let count = count as usize; // don't run this on 16 bit processors please :)
    let mut cards = game.card_list_for_position(side, CardPosition::DeckTop(side));
    let result = if count <= cards.len() {
        cards[0..count].to_vec()
    } else {
        let remaining = count - cards.len();
        let mut shuffled =
            random::cards_in_position(game, side, CardPosition::DeckUnknown(side), remaining);
        shuffled.append(&mut cards);
        shuffled
    };

    for card_id in &result {
        move_card(game, *card_id, CardPosition::DeckTop(side))?;
    }

    Ok(result)
}

/// Invoked after taking a game action to check if the turn should be switched
/// for the provided player.
pub fn check_end_turn(game: &mut GameState) -> Result<()> {
    if !matches!(game.data.phase, GamePhase::Play) {
        return Ok(());
    }

    let turn = game.data.turn;
    let side = turn.side;

    if game.player(side).actions == 0 && game.data.raid.is_none() {
        let max_hand_size = queries::maximum_hand_size(game, side) as usize;
        let hand = game.card_list_for_position(side, CardPosition::Hand(side));
        if hand.len() > max_hand_size {
            let count = hand.len() - max_hand_size;
            for card_id in hand.iter().take(count) {
                move_card(game, *card_id, CardPosition::DiscardPile(side))?;
            }
        }

        let turn_number = match side {
            Side::Overlord => turn.turn_number,
            Side::Champion => turn.turn_number + 1,
        };
        let next_side = side.opponent();
        start_turn(game, next_side, turn_number)?;
    }

    Ok(())
}

/// Increases the level of all `can_level_up` Overlord cards in a room by 1. If
/// a Scheme card's level reaches its `level_requirement`, that card is
/// immediately scored and moved to the Overlord score zone.
///
/// Does not spend mana/actions etc.
pub fn level_up_room(game: &mut GameState, room_id: RoomId) -> Result<()> {
    let occupants = game.card_list_for_position(
        Side::Overlord,
        CardPosition::Room(room_id, RoomLocation::Occupant),
    );
    let can_level = occupants
        .into_iter()
        .filter(|card_id| flags::can_level_up_card(game, *card_id))
        .collect::<Vec<_>>();

    for occupant_id in can_level {
        add_level_counters(game, occupant_id, 1)?;
    }

    Ok(())
}

/// Adds `amount` level counters to the provided card.
///
/// If the card has scheme points and the level requirement is met, the card is
/// immediately scored and moved to the Overlord's score zone.
///
/// Returns an error if this card cannot be leveled up.
pub fn add_level_counters(game: &mut GameState, card_id: CardId, amount: u32) -> Result<()> {
    verify!(flags::can_level_up_card(game, card_id));
    game.card_mut(card_id).data.card_level += amount;
    let card = game.card(card_id);
    if let Some(scheme_points) = crate::get(card.name).config.stats.scheme_points {
        if card.data.card_level >= scheme_points.level_requirement {
            game.card_mut(card_id).turn_face_up();
            move_card(game, card_id, CardPosition::Scoring)?;
            game.record_update(|| GameUpdate::ScoreCard(Side::Overlord, card_id));
            dispatch::invoke_event(game, OverlordScoreCardEvent(card_id))?;
            dispatch::invoke_event(
                game,
                ScoreCardEvent(ScoreCard { player: Side::Overlord, card_id }),
            )?;
            score_points(game, Side::Overlord, scheme_points.points)?;
            move_card(game, card_id, CardPosition::Scored(Side::Overlord))?;
        }
    }

    Ok(())
}

/// Attempt to pay a project's cost and turn it face up. Has no effect if the
/// card is not in play, already face up, or if the cost cannot be paid.
///
/// Returns true if the card was unveiled.
pub fn try_unveil_project(game: &mut GameState, card_id: CardId) -> Result<bool> {
    let result = if game.card(card_id).is_face_down() && game.card(card_id).position().in_play() {
        if let Some(custom_cost) = &crate::card_definition(game, card_id).cost.custom_cost {
            if (custom_cost.can_pay)(game, card_id) {
                (custom_cost.pay)(game, card_id)?;
            } else {
                return Ok(false);
            }
        }

        match queries::mana_cost(game, card_id) {
            None => {
                game.card_mut(card_id).turn_face_up();
                true
            }
            Some(cost)
                if cost <= mana::get(game, card_id.side, ManaPurpose::PayForCard(card_id)) =>
            {
                mana::spend(game, card_id.side, ManaPurpose::PayForCard(card_id), cost)?;
                game.card_mut(card_id).turn_face_up();
                true
            }
            _ => false,
        }
    } else {
        false
    };

    if result {
        game.record_update(|| GameUpdate::UnveilProject(card_id));
        dispatch::invoke_event(game, UnveilProjectEvent(card_id))?;
    }

    Ok(result)
}

/// Equivalent function to [try_unveil_project] which ignores costs.
pub fn unveil_project_for_free(game: &mut GameState, card_id: CardId) -> Result<bool> {
    let result = if game.card(card_id).is_face_down() && game.card(card_id).position().in_play() {
        game.card_mut(card_id).turn_face_up();
        true
    } else {
        false
    };

    if result {
        game.record_update(|| GameUpdate::UnveilProject(card_id));
        dispatch::invoke_event(game, UnveilProjectEvent(card_id))?;
    }

    Ok(result)
}

/// Starts the turn for the `next_side` player.
fn start_turn(game: &mut GameState, next_side: Side, turn_number: TurnNumber) -> Result<()> {
    game.data.phase = GamePhase::Play;
    game.data.turn = TurnData { side: next_side, turn_number };

    info!(?next_side, "start_player_turn");
    game.record_update(|| GameUpdate::StartTurn(next_side));

    if next_side == Side::Overlord {
        dispatch::invoke_event(game, DuskEvent(turn_number))?;
    } else {
        dispatch::invoke_event(game, DawnEvent(turn_number))?;
    }
    game.player_mut(next_side).actions = queries::start_of_turn_action_count(game, next_side);
    draw_cards(game, next_side, 1)?;
    Ok(())
}

/// Clears card state which is specific to a card being in play.
///
/// Automatically invoked by [move_card] when a card moves to a non-play zone.
fn clear_counters(game: &mut GameState, card_id: CardId) {
    game.card_mut(card_id).data.card_level = 0;
    game.card_mut(card_id).data.stored_mana = 0;
    game.card_mut(card_id).data.boost_count = 0;
}

/// Options when invoking [summon_minion]
#[derive(Eq, PartialEq, Debug)]
pub enum SummonMinion {
    PayCosts,
    IgnoreCosts,
}

/// Turn a minion card in play face up, paying its costs based on the
/// [SummonMinion] value provided.
///
/// Returns an error if the indicated card is already face-up.
pub fn summon_minion(game: &mut GameState, card_id: CardId, costs: SummonMinion) -> Result<()> {
    verify!(game.card(card_id).is_face_down());
    if costs == SummonMinion::PayCosts {
        if let Some(cost) = queries::mana_cost(game, card_id) {
            mana::spend(game, Side::Overlord, ManaPurpose::PayForCard(card_id), cost)?;
        }

        if let Some(custom_cost) = &crate::card_definition(game, card_id).cost.custom_cost {
            (custom_cost.pay)(game, card_id)?;
        }
    }

    dispatch::invoke_event(game, SummonMinionEvent(card_id))?;
    game.card_mut(card_id).turn_face_up();
    game.record_update(|| GameUpdate::SummonMinion(card_id));
    Ok(())
}

/// Deals damage. Discards random card from the hand of the Champion player. If
/// no cards remain, this player loses the game.
pub fn deal_damage(game: &mut GameState, source: impl HasAbilityId, amount: u32) -> Result<()> {
    let mut discarded = vec![];
    for _ in 0..amount {
        if let Some(card_id) =
            random::card_in_position(game, Side::Champion, CardPosition::Hand(Side::Champion))
        {
            move_card(game, card_id, CardPosition::DiscardPile(Side::Champion))?;
            discarded.push(card_id);
        } else {
            game_over(game, Side::Overlord)?;
        }
    }

    dispatch::invoke_event(
        game,
        DealtDamageEvent(DealtDamage { source: source.ability_id(), amount, discarded }),
    )?;

    Ok(())
}
