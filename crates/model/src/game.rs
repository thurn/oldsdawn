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
use crate::card_state::{AbilityState, CardPosition, CardState};
use crate::primitives::{
    AbilityId, AbilityIndex, ActionCount, CardId, ManaValue, PointsValue, RaidId, Side, TurnNumber,
};
use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;
use rand::{thread_rng, Rng, RngCore};
use std::cell::RefCell;
use std::collections::btree_map::Entry;
use std::iter::{Enumerate, Map};
use std::slice::Iter;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Default)]
pub struct PlayerState {
    pub mana: ManaValue,
    pub actions: ActionCount,
    pub score: PointsValue,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Default)]
pub struct AnimationBuffer {}

#[derive(Debug, Clone, Copy)]
pub struct RaidState {
    /// Unique ID for this raid
    pub raid_id: RaidId,
    /// Encounter position within this raid
    pub encounter_number: u32,
    /// Player who is next to act within this raid
    pub priority: Side,
}

#[derive(Debug, Clone)]
pub struct GameData {
    /// Current player whose turn it is
    pub turn: Side,
    /// Turn number for that player
    pub turn_number: TurnNumber,
    /// Data about an ongoing raid, if any
    pub raid: Option<RaidState>,
}

/// Stores the primary state for an ongoing game
#[derive(Debug, Clone)]
pub struct GameState {
    id: String,
    overlord_cards: Vec<CardState>,
    champion_cards: Vec<CardState>,
    rng: ThreadRng,
    overlord: PlayerState,
    champion: PlayerState,
    data: GameData,
    modified: bool,
    animations: AnimationBuffer,
}

impl GameState {
    pub fn new_game(
        id: String,
        overlord_deck: Vec<CardName>,
        champion_deck: Vec<CardName>,
    ) -> Self {
        let champion_start = overlord_deck.len();
        Self {
            id,
            overlord_cards: Self::make_deck(overlord_deck, Side::Overlord),
            champion_cards: Self::make_deck(champion_deck, Side::Champion),
            rng: thread_rng(),
            overlord: PlayerState::default(),
            champion: PlayerState::default(),
            data: GameData { turn: Side::Overlord, turn_number: 1, raid: None },
            modified: false,
            animations: AnimationBuffer::default(),
        }
    }

    /// Reset the value of 'modified' flags to false
    pub fn clear_modified_flags(&mut self) {
        self.modified = false;
    }

    /// Whether the player or game state has been modified since the last call to
    /// [Self::clear_modified_flags]
    pub fn modified(&self) -> bool {
        self.modified
    }

    /// ID for this game
    pub fn id(&self) -> &String {
        &self.id
    }

    /// All Card IDs present in this game
    pub fn card_ids(&self) -> impl Iterator<Item = CardId> {
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

    /// Random number generator to use for this game, can be set in tests for deterministic outcomes
    pub fn rng(&self) -> &ThreadRng {
        &self.rng
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
        self.modified = true;
        match side {
            Side::Overlord => &mut self.overlord,
            Side::Champion => &mut self.champion,
        }
    }

    /// Return a random card in the provided `position`, or None if there are no cards in that
    /// position
    pub fn random_card(&mut self, position: CardPosition) -> Option<CardId> {
        self.overlord_cards
            .iter()
            .chain(self.champion_cards.iter())
            .choose(&mut self.rng)
            .map(CardState::id)
    }

    /// Cards in a player's hand
    pub fn hand(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position().in_hand())
    }

    /// Mutable version of [Self::hand]
    pub fn hand_mut(&mut self, side: Side) -> impl Iterator<Item = &mut CardState> {
        self.cards_mut(side).iter_mut().filter(|c| c.position().in_hand())
    }

    /// Cards in a player's deck
    pub fn deck(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position().in_deck())
    }

    /// Mutable version of [Self::deck]
    pub fn deck_mut(&mut self, side: Side) -> impl Iterator<Item = &mut CardState> {
        self.cards_mut(side).iter_mut().filter(|c| c.position().in_deck())
    }

    /// Cards in a player's discard pile
    pub fn discard_pile(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position().in_discard_pile())
    }

    /// Mutable version of [Self::discard_pile]
    pub fn discard_pile_mut(&mut self, side: Side) -> impl Iterator<Item = &mut CardState> {
        self.cards_mut(side).iter_mut().filter(|c| c.position().in_discard_pile())
    }

    /// General game state
    pub fn data(&self) -> &GameData {
        &self.data
    }

    /// Mutable version of [Self::data]
    pub fn data_mut(&mut self) -> &mut GameData {
        self.modified = true;
        &mut self.data
    }

    /// Animations to send on the next client response
    pub fn animations(&self) -> &AnimationBuffer {
        &self.animations
    }

    /// Mutable version of [Self::animations]
    pub fn animations_mut(&mut self) -> &mut AnimationBuffer {
        &mut self.animations
    }

    /// Create card states for a deck
    fn make_deck(deck: Vec<CardName>, side: Side) -> Vec<CardState> {
        deck.into_iter()
            .enumerate()
            .map(move |(index, name)| {
                CardState::new(CardId::new(side, index), name, CardPosition::Deck(side))
            })
            .collect()
    }

    /// All cards in this game
    fn all_cards(&self) -> impl Iterator<Item = &CardState> {
        self.overlord_cards.iter().chain(self.champion_cards.iter())
    }

    /// Cards for a player
    fn cards(&self, side: Side) -> &Vec<CardState> {
        match side {
            Side::Overlord => &self.overlord_cards,
            Side::Champion => &self.champion_cards,
        }
    }

    /// Mutable version of [Self::cards]
    fn cards_mut(&mut self, side: Side) -> &mut Vec<CardState> {
        match side {
            Side::Overlord => &mut self.overlord_cards,
            Side::Champion => &mut self.champion_cards,
        }
    }
}
