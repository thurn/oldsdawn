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

use crate::card_state::{CardPosition, CardState};
use crate::primitives::{ActionCount, CardId, ManaValue, Score, Side};

#[derive(PartialEq, Eq, Hash, Debug, Clone, Default)]
pub struct PlayerState {
    pub mana: ManaValue,
    pub actions: ActionCount,
    pub score: Score,
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

#[derive(PartialEq, Eq, Hash, Debug, Clone, Default)]
pub struct GameState {
    cards: Vec<CardState>,
    pub overlord: OverlordState,
    pub champion: ChampionState,

    /** Optionally, animations to include in the next response to the client. */
    pub animations: Option<AnimationBuffer>,
}

impl GameState {
    pub fn card(&self, card_id: CardId) -> &CardState {
        &self.cards[card_id.0]
    }

    pub fn card_mut(&mut self, card_id: CardId) -> &mut CardState {
        &mut self.cards[card_id.0]
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

    pub fn cards_in_position(&self, position: CardPosition) -> impl Iterator<Item = &CardState> {
        self.cards.iter().filter(move |card| card.position == position)
    }

    pub fn cards_in_position_mut(
        &mut self,
        position: CardPosition,
    ) -> impl Iterator<Item = &mut CardState> {
        self.cards.iter_mut().filter(move |card| card.position == position)
    }

    pub fn hand(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards_in_position(CardPosition::Hand(side))
    }

    pub fn hand_mut(&mut self, side: Side) -> impl Iterator<Item = &mut CardState> {
        self.cards_in_position_mut(CardPosition::Hand(side))
    }

    pub fn deck(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards_in_position(CardPosition::Deck(side))
    }

    pub fn deck_mut(&mut self, side: Side) -> impl Iterator<Item = &mut CardState> {
        self.cards_in_position_mut(CardPosition::Deck(side))
    }

    pub fn discard_pile(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards_in_position(CardPosition::DiscardPile(side))
    }

    pub fn discard_pile_mut(&mut self, side: Side) -> impl Iterator<Item = &mut CardState> {
        self.cards_in_position_mut(CardPosition::DiscardPile(side))
    }
}
