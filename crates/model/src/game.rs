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
    AbilityId, AbilityIndex, ActionCount, CardId, EncounterId, ManaValue, PointsValue, Side,
    TurnNumber,
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
pub struct OverlordState {
    pub state: PlayerState,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Default)]
pub struct ChampionState {
    pub state: PlayerState,
}

#[derive(PartialEq, Eq, Hash, Debug, Clone, Default)]
pub struct AnimationBuffer {}

/// Stores the primary state for an ongoing game
#[derive(Debug, Clone, Default)]
pub struct GameState {
    /// Card states
    overlord_cards: Vec<CardState>,
    champion_cards: Vec<CardState>,
    /// Random number generator to use for this game, can be set in tests for deterministic outcomes
    pub rng: ThreadRng,
    /// Overlord player state
    pub overlord: OverlordState,
    /// Champion player state
    pub champion: ChampionState,
    /// Current game turn
    pub turn_number: TurnNumber,
    /// Current raid state, if any
    pub active_raid: Option<EncounterId>,
    /// Animations to send on the next client response
    pub animations: Option<AnimationBuffer>,
}

impl GameState {
    pub fn new(overlord_deck: Vec<CardName>, champion_deck: Vec<CardName>) -> Self {
        let champion_start = overlord_deck.len();
        Self {
            overlord_cards: Self::make_deck(overlord_deck, Side::Overlord),
            champion_cards: Self::make_deck(champion_deck, Side::Champion),
            rng: thread_rng(),
            overlord: OverlordState::default(),
            champion: ChampionState::default(),
            turn_number: 0,
            active_raid: None,
            animations: None,
        }
    }

    fn make_deck(deck: Vec<CardName>, side: Side) -> Vec<CardState> {
        deck.into_iter()
            .enumerate()
            .map(move |(index, name)| {
                CardState::new(CardId::new(side, index), name, CardPosition::Deck(side))
            })
            .collect()
    }

    fn all_cards(&self) -> impl Iterator<Item = &CardState> {
        self.overlord_cards.iter().chain(self.champion_cards.iter())
    }

    fn cards(&self, side: Side) -> &Vec<CardState> {
        match side {
            Side::Overlord => &self.overlord_cards,
            Side::Champion => &self.champion_cards,
        }
    }

    fn cards_mut(&mut self, side: Side) -> &mut Vec<CardState> {
        match side {
            Side::Overlord => &mut self.overlord_cards,
            Side::Champion => &mut self.champion_cards,
        }
    }

    pub fn card_ids(&self) -> impl Iterator<Item = CardId> {
        (0..self.overlord_cards.len())
            .map(|index| CardId::new(Side::Overlord, index))
            .chain((0..self.champion_cards.len()).map(|index| CardId::new(Side::Champion, index)))
    }

    pub fn card(&self, card_id: impl Into<CardId>) -> &CardState {
        let id = card_id.into();
        &self.cards(id.side)[id.index]
    }

    pub fn card_mut(&mut self, card_id: impl Into<CardId>) -> &mut CardState {
        let id = card_id.into();
        &mut self.cards_mut(id.side)[id.index]
    }

    pub fn player_state(&self, side: Side) -> &PlayerState {
        match side {
            Side::Overlord => &self.overlord.state,
            Side::Champion => &self.champion.state,
        }
    }

    pub fn player_state_mut(&mut self, side: Side) -> &mut PlayerState {
        match side {
            Side::Overlord => &mut self.overlord.state,
            Side::Champion => &mut self.champion.state,
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

    pub fn hand(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position.in_hand())
    }

    pub fn hand_mut(&mut self, side: Side) -> impl Iterator<Item = &mut CardState> {
        self.cards_mut(side).iter_mut().filter(|c| c.position.in_hand())
    }

    pub fn deck(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position.in_deck())
    }

    pub fn deck_mut(&mut self, side: Side) -> impl Iterator<Item = &mut CardState> {
        self.cards_mut(side).iter_mut().filter(|c| c.position.in_deck())
    }

    pub fn discard_pile(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position.in_discard_pile())
    }

    pub fn discard_pile_mut(&mut self, side: Side) -> impl Iterator<Item = &mut CardState> {
        self.cards_mut(side).iter_mut().filter(|c| c.position.in_discard_pile())
    }

    pub fn ability(&self, ability_id: AbilityId) -> Option<&AbilityState> {
        self.card(ability_id.card_id).abilities.get(&ability_id.index)
    }

    pub fn ability_mut(&mut self, ability_id: AbilityId) -> Entry<AbilityIndex, AbilityState> {
        self.card_mut(ability_id.card_id).abilities.entry(ability_id.index)
    }
}
