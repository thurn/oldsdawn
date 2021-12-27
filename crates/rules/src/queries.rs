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

use data::card_definition::CardStats;
use data::card_state::CardPosition;
use data::delegates::{
    ActionCostQuery, AttackValueQuery, BoostCountQuery, CanPlayCardQuery, Flag,
    HealthValueQuery, ManaCostQuery, ShieldValueQuery,
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
pub fn stats(game: &GameState, card_id: impl Into<CardId>) -> &CardStats {
    &crate::get(game.card(card_id).name).config.stats
}

/// Returns whether a given card can currently be played
pub fn can_play(game: &GameState, side: Side, card_id: CardId) -> bool {
    let can_play = in_main_phase(game, side)
        && side == card_id.side
        && matches!(mana_cost(game, card_id), Some(cost) if cost <= game.player(side).mana);
    dispatch::perform_query(game, CanPlayCardQuery(card_id), Flag::new(can_play)).into()
}

pub fn mana_cost(game: &GameState, card_id: CardId) -> Option<ManaValue> {
    dispatch::perform_query(
        game,
        ManaCostQuery(card_id),
        crate::get(game.card(card_id).name).cost.mana,
    )
}

pub fn action_cost(game: &GameState, card_id: impl Into<CardId> + Copy) -> ActionCount {
    dispatch::perform_query(
        game,
        ActionCostQuery(card_id.into()),
        crate::get(game.card(card_id).name).cost.actions,
    )
}

pub fn attack(game: &GameState, card_id: impl Into<CardId> + Copy) -> AttackValue {
    dispatch::perform_query(
        game,
        AttackValueQuery(card_id.into()),
        stats(game, card_id).base_attack.unwrap_or(0),
    )
}

pub fn health(game: &GameState, card_id: impl Into<CardId> + Copy) -> HealthValue {
    dispatch::perform_query(
        game,
        HealthValueQuery(card_id.into()),
        stats(game, card_id).health.unwrap_or(0),
    )
}

pub fn shield(game: &GameState, card_id: impl Into<CardId> + Copy) -> ShieldValue {
    dispatch::perform_query(
        game,
        ShieldValueQuery(card_id.into()),
        stats(game, card_id).shield.unwrap_or(0),
    )
}

pub fn boost_count(game: &GameState, card_id: impl Into<CardId> + Copy) -> BoostCount {
    dispatch::perform_query(
        game,
        BoostCountQuery(card_id.into()),
        game.card(card_id).data.boost_count,
    )
}

// Returns true if the provided `side` player is currently in their Main phase,
// i.e. that it is their turn, that they have action points available, that a
// raid is not currently ongoing, that we are not currently waiting for an
// interface prompt response, etc.
pub fn in_main_phase(game: &GameState, side: Side) -> bool {
    game.player(side).actions > 0 && game.data.turn == side && game.data.raid.is_none()
}
