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

//! Core functions for querying the current state of a game

use data::card_definition::CardStats;
use data::card_state::CardPosition;
use data::delegates::{
    ActionCostQuery, AttackValueQuery, BoostCountQuery, HealthValueQuery, ManaCostQuery,
    ShieldValueQuery, StartOfTurnActionsQuery,
};
use data::game::GameState;
use data::primitives::{
    ActionCount, AttackValue, BoostCount, CardId, HealthValue, ManaValue, ShieldValue, Side,
};

use crate::dispatch;

/// Returns the top card of the indicated player's deck, selecting randomly if
/// no cards are known to be present there. Returns None if the deck is empty.
pub fn top_of_deck(game: &GameState, side: Side) -> Option<CardId> {
    game.cards(side)
        .iter()
        .filter(|c| c.position == CardPosition::DeckTop(side))
        .max_by_key(|c| c.sorting_key)
        .map_or_else(|| game.random_card(CardPosition::DeckUnknown(side)), |card| Some(card.id))
}

/// Obtain the [CardStats] for a given card
pub fn stats(game: &GameState, card_id: CardId) -> &CardStats {
    &crate::get(game.card(card_id).name).config.stats
}

/// Returns the mana cost for a given card, if any
pub fn mana_cost(game: &GameState, card_id: CardId) -> Option<ManaValue> {
    dispatch::perform_query(
        game,
        ManaCostQuery(card_id),
        crate::get(game.card(card_id).name).cost.mana,
    )
}

/// Returns the action point cost for a given card
pub fn action_cost(game: &GameState, card_id: CardId) -> ActionCount {
    dispatch::perform_query(
        game,
        ActionCostQuery(card_id),
        crate::get(game.card(card_id).name).cost.actions,
    )
}

/// Returns the attack power value for a given card, or 0 by default.
pub fn attack(game: &GameState, card_id: CardId) -> AttackValue {
    dispatch::perform_query(
        game,
        AttackValueQuery(card_id),
        stats(game, card_id).base_attack.unwrap_or(0),
    )
}

/// Returns the health value for a given card, or 0 by default.
pub fn health(game: &GameState, card_id: CardId) -> HealthValue {
    dispatch::perform_query(
        game,
        HealthValueQuery(card_id),
        stats(game, card_id).health.unwrap_or(0),
    )
}

/// Returns the shield value for a given card, or 0 by default.
pub fn shield(game: &GameState, card_id: CardId) -> ShieldValue {
    dispatch::perform_query(
        game,
        ShieldValueQuery(card_id),
        stats(game, card_id).shield.unwrap_or(0),
    )
}

/// Returns the [BoostCount] for a given card.
pub fn boost_count(game: &GameState, card_id: CardId) -> BoostCount {
    dispatch::perform_query(game, BoostCountQuery(card_id), game.card(card_id).data.boost_count)
}

/// Returns the amount of mana the owner of `card_id` would need to spend to
/// raise its [AttackValue] to the provided `target` by activating boosts.
/// Returns 0 if this card's attack value already meets the target. Returns None
/// if this card does not have a defined boost ability.
pub fn boost_target_mana_cost(
    game: &GameState,
    card_id: CardId,
    target: AttackValue,
) -> Option<ManaValue> {
    let current = attack(game, card_id);
    if current >= target {
        Some(0)
    } else if let Some(boost) = crate::card_definition(game, card_id).config.stats.attack_boost {
        assert!(boost.bonus > 0);
        let increase = target - current;
        // If the boost does not evenly divide into the target, we need to apply it an
        // additional time.
        let add = if (increase % boost.bonus) == 0 { 0 } else { 1 };

        #[allow(clippy::integer_division)] // Deliberate integer truncation
        Some((add + (increase / boost.bonus)) * boost.cost)
    } else {
        None
    }
}

/// Returns true if the provided `side` player is currently in their Main phase
/// with no pending prompt responses, and thus can take a primary game action.
pub fn in_main_phase(game: &GameState, side: Side) -> bool {
    game.player(side).actions > 0
        && game.data.turn == side
        && game.data.raid.is_none()
        && game.overlord.prompt.is_none()
        && game.champion.prompt.is_none()
}

/// Look up the number of action points a player receives at the start of their
/// turn
pub fn start_of_turn_action_count(game: &GameState, side: Side) -> ActionCount {
    dispatch::perform_query(game, StartOfTurnActionsQuery(side), 3)
}
