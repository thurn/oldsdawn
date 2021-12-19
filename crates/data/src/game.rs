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
use crate::card_state::{AbilityState, CardPosition, CardPositionTypes, CardState, SortingKey};
use crate::deck::Deck;
use crate::primitives::{
    AbilityId, AbilityIndex, ActionCount, CardId, GameId, ManaValue, PointsValue, RaidId, Side,
    TurnNumber,
};
use rand::rngs::ThreadRng;
use rand::seq::IteratorRandom;
use rand::{thread_rng, Rng, RngCore};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::btree_map::Entry;
use std::iter;
use std::iter::{Enumerate, Map};
use std::slice::Iter;

#[derive(PartialEq, Eq, Hash, Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlayerState {
    pub mana: ManaValue,
    pub actions: ActionCount,
    pub score: PointsValue,
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
    /// Whether animations should be produced for this game
    pub enable_animations: bool,
}

/// Stores the primary state for an ongoing game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    id: GameId,
    overlord_cards: Vec<CardState>,
    champion_cards: Vec<CardState>,
    overlord: PlayerState,
    champion: PlayerState,
    data: GameData,
    modified: bool,
    animations: Option<AnimationBuffer>,
    next_sorting_key: SortingKey,
}

impl GameState {
    /// Creates a new game with the provided `id` and identity rules and decks for both players
    pub fn new_game(
        id: GameId,
        overlord_deck: Deck,
        champion_deck: Deck,
        options: NewGameOptions,
    ) -> Self {
        Self {
            id,
            overlord_cards: Self::make_deck(overlord_deck, Side::Overlord),
            champion_cards: Self::make_deck(champion_deck, Side::Champion),
            overlord: PlayerState::default(),
            champion: PlayerState::default(),
            data: GameData { turn: Side::Overlord, turn_number: 1, raid: None },
            modified: false,
            animations: options.enable_animations.then(AnimationBuffer::default),
            next_sorting_key: 1,
        }
    }

    /// Reset the value of the 'modified' flag to false
    pub fn clear_modified_flag(&mut self) {
        self.modified = false;
    }

    /// Whether the player or game state has been modified since the last call to
    /// [Self::clear_modified_flag]
    pub fn modified(&self) -> bool {
        self.modified
    }

    /// ID for this game
    pub fn id(&self) -> GameId {
        self.id
    }

    /// Returns the identity card for the provided Side.
    ///
    /// It is an error for there to be zero or multiple rules in the `Identity` card position. If
    /// this does occur, this method will panic (in the case of zero rules) or return an arbitrary
    /// identity card (in the case of multiples).
    pub fn identity(&self, side: Side) -> &CardState {
        self.cards(side)
            .iter()
            .find(|c| CardPositionTypes::Identity == c.position().into())
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

    /// Moves a card to a new [CardPosition].
    pub fn move_card(&mut self, card_id: impl Into<CardId>, new_position: CardPosition) {
        let key = self.next_sorting_key;
        self.card_mut(card_id).move_to(new_position, key);
        self.next_sorting_key += 1;
    }

    /// Finds the [CardPosition] of a given card.
    pub fn card_position(&self, card_id: impl Into<CardId>) -> CardPosition {
        self.card(card_id).position()
    }

    /// Return a random card in the provided `position`, or None if there are no rules in that
    /// position
    pub fn random_card(&mut self, position: CardPosition) -> Option<CardId> {
        self.overlord_cards
            .iter()
            .chain(self.champion_cards.iter())
            .choose(&mut rand::thread_rng())
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

    /// Animations to send on the next client response. If animations are currently disabled,
    /// will return None.
    pub fn animations(&self) -> &Option<AnimationBuffer> {
        &self.animations
    }

    /// Mutable version of [Self::animations]
    pub fn animations_mut(&mut self) -> &mut Option<AnimationBuffer> {
        &mut self.animations
    }

    /// Create card states for a deck
    fn make_deck(deck: Deck, side: Side) -> Vec<CardState> {
        deck.card_names()
            .iter()
            .enumerate()
            .map(move |(index, name)| CardState::new(CardId::new(side, index), *name, side))
            .collect()
    }

    /// All rules in this game
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

    /// Mutable version of [Self::rules]
    fn cards_mut(&mut self, side: Side) -> &mut Vec<CardState> {
        match side {
            Side::Overlord => &mut self.overlord_cards,
            Side::Champion => &mut self.champion_cards,
        }
    }
}
