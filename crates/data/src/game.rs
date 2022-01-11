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

//! Core data structures for tracking the state of an ongoing game.

use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};

use crate::card_state::{CardPosition, CardPositionKind, CardState, SortingKey};
use crate::deck::Deck;
use crate::primitives::{
    ActionCount, CardId, GameId, ItemLocation, ManaValue, PlayerId, PointsValue, RaidId, RoomId,
    RoomLocation, Side, TurnNumber,
};
use crate::prompt::Prompt;
use crate::updates::UpdateTracker;

/// State of a player within a game, containing their score and available
/// resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: PlayerId,
    pub mana: ManaValue,
    pub actions: ActionCount,
    pub score: PointsValue,
    /// A choice this player is currently facing. Automatically cleared when
    /// a [Prompt] response is received.
    pub prompt: Option<Prompt>,
}

impl PlayerState {
    /// Create the default player state for a new game
    pub fn new_game(id: PlayerId, actions: ActionCount) -> Self {
        Self { id, mana: 5, actions, score: 0, prompt: None }
    }
}

/// Identifies steps within a raid
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum RaidPhase {
    /// The raid has started and the Overlord is deciding whether to activate
    /// the target room.
    Activation,
    /// The defender with the provided ordinal position is currently being
    /// encountered. The Champion is deciding which weapons, if any, to employ.
    Encounter(usize),
    /// The Champion has defeated the defender with the provided ordinal
    /// position and is deciding whether to continue to the next defender or
    /// retreat.
    Continue(usize),
    /// The Champion has bypassed all of the defenders for this room and is now
    /// accessing its contents
    Access,
}

/// Data about an active raid
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RaidData {
    /// Unique ID for this raid
    pub raid_id: RaidId,
    /// Room being targeted by this raid
    pub target: RoomId,
    /// Current phase within this raid
    pub phase: RaidPhase,
    /// True if the Overlord activated this room in response to the raid.
    ///
    /// Initially false if the activation decision has not been made yet.
    pub active: bool,
}

/// Describes options for this game & the set of rules it is using.
#[derive(Debug, Clone, Default, Copy, Serialize, Deserialize)]
pub struct GameConfiguration {
    /// If true, all random choices within this game will be made
    /// deterministically instead of using a random number generator. Useful for
    /// e.g. unit tests.
    pub deterministic: bool,
    /// Whether to run in simulation mode and thus disable update tracking
    pub simulation: bool,
}

/// State and configuration of the overall game, including whose turn it is and
/// whether a raid is active.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct GameData {
    /// Current player whose turn it is
    pub turn: Side,
    /// Turn number for that player
    pub turn_number: TurnNumber,
    /// Data about an ongoing raid, if any
    pub raid: Option<RaidData>,
    /// Counter to create unique IDs for raids within this game
    pub next_raid_id: u32,
    /// Game options
    pub config: GameConfiguration,
}

/// Stores the primary state for an ongoing game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Unique identifier for this game
    pub id: GameId,
    /// General game state & configuration
    pub data: GameData,
    /// Used to track changes to game state in order to update the client. Code
    /// which mutates the game state is responsible for appending a
    /// description of the change to `updates` via [UpdateTracker::push].
    ///
    /// A new `updates` buffer should be set for each network request to track
    /// changes in response to that request. Consequently, its value is not
    /// serialized.
    #[serde(skip)]
    pub updates: UpdateTracker,
    /// Cards for the overlord player. In general, code should use one of the
    /// helper methods below instead of accessing this directly.
    pub overlord_cards: Vec<CardState>,
    /// Cards for the champion player. In general, code should use one of the
    /// helper methods below instead of accessing this directly.
    pub champion_cards: Vec<CardState>,
    /// State for the overlord player
    pub overlord: PlayerState,
    /// State for the champion player
    pub champion: PlayerState,
    /// Next sorting key to use for card moves. Automatically updated by
    /// [Self::next_sorting_key] and [Self::move_card].
    next_sorting_key: SortingKey,
}

impl GameState {
    /// Creates a new game with the provided [GameId] and decks for both players
    pub fn new_game(
        id: GameId,
        overlord_deck: Deck,
        champion_deck: Deck,
        config: GameConfiguration,
    ) -> Self {
        Self {
            id,
            overlord_cards: Self::make_deck(&overlord_deck, Side::Overlord),
            champion_cards: Self::make_deck(&champion_deck, Side::Champion),
            overlord: PlayerState::new_game(overlord_deck.owner_id, 3 /* actions */),
            champion: PlayerState::new_game(champion_deck.owner_id, 0 /* actions */),
            data: GameData {
                turn: Side::Overlord,
                turn_number: 1,
                raid: None,
                next_raid_id: 1,
                config,
            },
            updates: UpdateTracker::new(!config.simulation),
            next_sorting_key: 1,
        }
    }

    /// Returns the identity card for the provided Side.
    ///
    /// Panics if no identity card is present for this player.
    pub fn identity(&self, side: Side) -> &CardState {
        self.cards(side)
            .iter()
            .find(|c| c.position.kind() == CardPositionKind::Identity)
            .expect("Identity Card")
    }

    /// Look up [CardState] for a card. Panics if this card is not present in
    /// the game.
    pub fn card(&self, card_id: CardId) -> &CardState {
        &self.cards(card_id.side)[card_id.index]
    }

    /// Mutable version of [Self::card]
    pub fn card_mut(&mut self, card_id: CardId) -> &mut CardState {
        &mut self.cards_mut(card_id.side)[card_id.index]
    }

    /// Cards for a player, in alphabetical order
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

    /// Returns a monotonically-increasing sorting key for object positions in
    /// this game.
    pub fn next_sorting_key(&mut self) -> u32 {
        let result = self.next_sorting_key;
        self.next_sorting_key += 1;
        result
    }

    /// Moves a card to a new [CardPosition], updating its sorting key.
    pub fn move_card(&mut self, card_id: CardId, new_position: CardPosition) {
        let key = self.next_sorting_key();
        let mut card = self.card_mut(card_id);
        card.position = new_position;
        card.sorting_key = key;
    }

    /// Return a random card in the provided `position`, or None if there are no
    /// cards in that position
    pub fn random_card(&self, position: CardPosition) -> Option<CardId> {
        let mut cards = self.all_cards().filter(|c| c.position == position);
        if self.data.config.deterministic {
            cards.next()
        } else {
            cards.choose(&mut rand::thread_rng())
        }
        .map(|c| c.id)
    }

    /// Cards owned by a given player in a given position, in alphabetical order
    pub fn cards_in_position(
        &self,
        side: Side,
        position: CardPosition,
    ) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(move |c| c.position == position)
    }

    /// Cards in a player's hand, in alphabetical order
    pub fn hand(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position.in_hand())
    }

    /// Cards in a player's discard pile, in alphabetical order
    pub fn discard_pile(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position.in_discard_pile())
    }

    /// Cards in a player's deck, in alphabetical order
    pub fn deck(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position.in_deck())
    }

    /// Returns true if this room has at least one hidden defender.
    pub fn has_hidden_defenders(&self, room_id: RoomId) -> bool {
        self.overlord_cards.iter().any(|c| {
            c.position == CardPosition::Room(room_id, RoomLocation::Defender) && !c.data.revealed
        })
    }

    /// Overlord cards defending a given room, in sorting-key order
    pub fn defender_list(&self, room_id: RoomId) -> Vec<&CardState> {
        let mut result = self
            .cards_in_position(Side::Overlord, CardPosition::Room(room_id, RoomLocation::Defender))
            .collect::<Vec<_>>();
        result.sort();
        result
    }

    /// Overlord cards in a given room, in alphabetical order
    pub fn occupants(&self, room_id: RoomId) -> impl Iterator<Item = &CardState> {
        self.cards_in_position(Side::Overlord, CardPosition::Room(room_id, RoomLocation::Occupant))
    }

    /// Champion cards which have been played as weapons, in alphabetical order
    pub fn weapons(&self) -> impl Iterator<Item = &CardState> {
        self.cards_in_position(Side::Champion, CardPosition::ArenaItem(ItemLocation::Weapons))
    }

    /// Champion cards which have been played as artifacts, in alphabetical
    /// order
    pub fn artifacts(&self) -> impl Iterator<Item = &CardState> {
        self.cards_in_position(Side::Champion, CardPosition::ArenaItem(ItemLocation::Artifacts))
    }

    /// All Card IDs present in this game
    pub fn all_card_ids(&self) -> impl Iterator<Item = CardId> {
        (0..self.overlord_cards.len())
            .map(|index| CardId::new(Side::Overlord, index))
            .chain((0..self.champion_cards.len()).map(|index| CardId::new(Side::Champion, index)))
    }

    /// All cards in this game.
    ///
    /// Overlord cards in alphabetical order followed by Champion cards in
    /// alphabetical order.
    pub fn all_cards(&self) -> impl Iterator<Item = &CardState> {
        self.overlord_cards.iter().chain(self.champion_cards.iter())
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
}
