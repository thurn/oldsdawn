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

use rand::prelude::{IteratorRandom, SliceRandom};

use crate::card_state::CardPosition;
use crate::game::GameState;
use crate::primitives::{CardId, Side};

/// Return a randomly-selected [CardId] of cards owned by the `side` player in
/// the given [CardPosition], or None if no such card exists.
#[allow(clippy::needless_collect)] // Invalid clippy warning
pub fn card_in_position(
    game: &mut GameState,
    side: Side,
    position: CardPosition,
) -> Option<CardId> {
    if game.rng.is_some() {
        let cards = game.cards_in_position(side, position).map(|c| c.id).collect::<Vec<_>>();
        cards.into_iter().choose(game.rng.as_mut().expect("rng"))
    } else {
        game.cards_in_position(side, position).choose(&mut rand::thread_rng()).map(|c| c.id)
    }
}

/// Return a vector of up to `count` randomly-selected [CardId]s of cards owned
/// by the `side` player in the given [CardPosition].
#[allow(clippy::needless_collect)] // Invalid clippy warning
pub fn cards_in_position(
    game: &mut GameState,
    side: Side,
    position: CardPosition,
    count: usize,
) -> Vec<CardId> {
    // Per the documentation of rand::choose_multiple(), "although the elements are
    // selected randomly, their order is not fully random. If random ordering is
    // desired, shuffle the result."
    if game.rng.is_some() {
        let cards = game.cards_in_position(side, position).map(|c| c.id).collect::<Vec<_>>();
        let rng = game.rng.as_mut().expect("rng");
        let mut result = cards.into_iter().choose_multiple(rng, count);
        result.shuffle(rng);
        result
    } else {
        let mut result = game
            .cards_in_position(side, position)
            .map(|c| c.id)
            .choose_multiple(&mut rand::thread_rng(), count);
        result.shuffle(&mut rand::thread_rng());
        result
    }
}

/// Given an iterator, return a randomly-selected value from this iterator using
/// the game random number generator.
///
/// Note that the iterator cannot be over references into the game itself, since
/// that would cause a borrow checker conflict.
pub fn choose<I>(game: &mut GameState, iterator: I) -> Option<I::Item>
where
    I: Iterator,
{
    if game.rng.is_some() {
        iterator.choose(game.rng.as_mut().expect("rng"))
    } else {
        iterator.choose(&mut rand::thread_rng())
    }
}
