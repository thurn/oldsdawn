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

use crate::card_name::CardName;
use crate::card_state;
use crate::card_state::{AbilityState, CardPosition, CardPositionKind, CardState, SortingKey};
use crate::deck::Deck;
use crate::primitives::{
    AbilityId, AbilityIndex, ActionCount, CardId, GameId, ManaValue, PointsValue, RaidId, Side,
    TurnNumber, UserId,
};
use crate::updates::{GameUpdate, UpdateTracker};
use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;
use rand::{thread_rng, Rng, RngCore};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::btree_map::Entry;
use std::iter;
use std::iter::{Enumerate, Map};
use std::slice::Iter;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: UserId,
    pub mana: ManaValue,
    pub actions: ActionCount,
    pub score: PointsValue,
}

impl PlayerState {
    pub fn new_game(id: UserId, actions: ActionCount) -> PlayerState {
        PlayerState { id, mana: 5, actions, score: 0 }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnimationBuffer {}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RaidState {
    /// Unique ID for this raid
    pub raid_id: RaidId,
    /// Encounter position within this raid
    pub encounter_number: u32,
    /// Player who is next to act within this raid
    pub priority: Side,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GameData {
    /// Current player whose turn it is
    pub turn: Side,
    /// Turn number for that player
    pub turn_number: TurnNumber,
    /// Data about an ongoing raid, if any
    pub raid: Option<RaidState>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NewGameOptions {
    /// Whether to run in simulation mode and thus disable update tracking
    pub simulation: bool,
}

/// Stores the primary state for an ongoing game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Unique identifier for this game
    pub id: GameId,
    /// General game state
    pub data: GameData,
    /// Used to track changes to game state in order to update the client. Code which mutates the
    /// game state is responsible for appending a description of the change to `updates` via
    /// [UpdateTracker::push].
    ///
    /// A new `updates` buffer should be set for each network request to track changes in response
    /// to that request. Consequently, its value is not serialized.
    #[serde(skip)]
    pub updates: UpdateTracker,
    /// Cards for the overlord player. In general, code should use one of the helper methods below
    /// instead of accessing this directly.
    pub overlord_cards: Vec<CardState>,
    /// Cards for the champion player. In general, code should use one of the helper methods below
    /// instead of accessing this directly.
    pub champion_cards: Vec<CardState>,
    /// State for the overlord player
    pub overlord: PlayerState,
    /// State for the champion player
    pub champion: PlayerState,
    /// Next sorting key to use for card moves. Automatically updated by [Self::move_card], do not
    /// mutate this directly.
    pub next_sorting_key: SortingKey,
}

impl GameState {
    /// Creates a new game with the provided `id` and decks for both players
    pub fn new_game(
        id: GameId,
        overlord_deck: Deck,
        champion_deck: Deck,
        options: NewGameOptions,
    ) -> Self {
        Self {
            id,
            overlord_cards: Self::make_deck(&overlord_deck, Side::Overlord),
            champion_cards: Self::make_deck(&champion_deck, Side::Champion),
            overlord: PlayerState::new_game(overlord_deck.owner_id, 3 /* actions */),
            champion: PlayerState::new_game(champion_deck.owner_id, 0 /* actions */),
            data: GameData { turn: Side::Overlord, turn_number: 1, raid: None },
            updates: if options.simulation {
                UpdateTracker::default()
            } else {
                UpdateTracker { update_list: Some(vec![]) }
            },
            next_sorting_key: 1,
        }
    }

    /// Returns the identity card for the provided Side.
    ///
    /// It is an error for there to be zero or multiple cards in the `Identity` card position. If
    /// this does occur, this method will panic (in the case of zero cards) or return an arbitrary
    /// identity card (in the case of multiples).
    pub fn identity(&self, side: Side) -> &CardState {
        self.cards(side)
            .iter()
            .find(|c| c.position.kind() == CardPositionKind::Identity)
            .expect("Identity Card")
    }

    /// All Card IDs present in this game
    pub fn all_card_ids(&self) -> impl Iterator<Item = CardId> {
        (0..self.overlord_cards.len())
            .map(|index| CardId::new(Side::Overlord, index))
            .chain((0..self.champion_cards.len()).map(|index| CardId::new(Side::Champion, index)))
    }

    /// Look up [CardState] for a card
    pub fn card(&self, card_id: impl Into<CardId>) -> &CardState {
        let id = card_id.into();
        &self.cards(id.side)[id.index]
    }

    /// Mutable version of [Self::card]
    pub fn card_mut(&mut self, card_id: impl Into<CardId>) -> &mut CardState {
        let id = card_id.into();
        &mut self.cards_mut(id.side)[id.index]
    }

    /// Cards for a player
    pub fn cards(&self, side: Side) -> &Vec<CardState> {
        match side {
            Side::Overlord => &self.overlord_cards,
            Side::Champion => &self.champion_cards,
        }
    }

    /// Mutable version of [Self::cards]
    pub fn cards_mut(&mut self, side: Side) -> &mut Vec<CardState> {
        match side {
            Side::Overlord => &mut self.overlord_cards,
            Side::Champion => &mut self.champion_cards,
        }
    }

    /// State for the players in the game
    pub fn player(&self, side: Side) -> &PlayerState {
        match side {
            Side::Overlord => &self.overlord,
            Side::Champion => &self.champion,
        }
    }

    /// Mutable version of [Self::player]
    pub fn player_mut(&mut self, side: Side) -> &mut PlayerState {
        match side {
            Side::Overlord => &mut self.overlord,
            Side::Champion => &mut self.champion,
        }
    }

    /// Moves a card to a new [CardPosition].
    pub fn move_card(&mut self, card_id: impl Into<CardId>, new_position: CardPosition) {
        let key = self.next_sorting_key;
        let mut card = self.card_mut(card_id);
        card.position = new_position;
        card.sorting_key = key;
        self.next_sorting_key += 1;
    }

    /// Return a random card in the provided `position`, or None if there are no cards in that
    /// position
    pub fn random_card(&self, position: CardPosition) -> Option<CardId> {
        self.overlord_cards
            .iter()
            .chain(self.champion_cards.iter())
            .choose(&mut rand::thread_rng())
            .map(|c| c.id)
    }

    /// Cards in a player's hand
    pub fn hand(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position.in_hand())
    }

    /// Mutable version of [Self::hand]
    pub fn hand_mut(&mut self, side: Side) -> impl Iterator<Item = &mut CardState> {
        self.cards_mut(side).iter_mut().filter(|c| c.position.in_hand())
    }

    /// Cards in a player's discard pile
    pub fn discard_pile(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position.in_discard_pile())
    }

    /// Mutable version of [Self::discard_pile]
    pub fn discard_pile_mut(&mut self, side: Side) -> impl Iterator<Item = &mut CardState> {
        self.cards_mut(side).iter_mut().filter(|c| c.position.in_discard_pile())
    }

    /// Create card states for a deck
    fn make_deck(deck: &Deck, side: Side) -> Vec<CardState> {
        let mut result = vec![CardState::new(
            CardId::new(side, 0),
            deck.identity,
            side,
            true, /* is_identity */
        )];

        result.extend(deck.card_names().iter().enumerate().map(move |(index, name)| {
            CardState::new(CardId::new(side, index + 1), *name, side, false /* is_identity */)
        }));

        result
    }

    /// All cards in this game
    pub fn all_cards(&self) -> impl Iterator<Item = &CardState> {
        self.overlord_cards.iter().chain(self.champion_cards.iter())
    }
}
