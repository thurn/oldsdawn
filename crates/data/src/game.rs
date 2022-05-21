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

#![allow(clippy::use_self)] // Required to use EnumKind

use std::collections::HashMap;

use anyhow::Result;
use enum_kinds::EnumKind;
use rand::seq::IteratorRandom;
use serde::{Deserialize, Serialize};

use crate::agent_definition::AgentData;
use crate::card_state::{AbilityState, CardPosition, CardPositionKind, CardState};
use crate::deck::Deck;
use crate::delegates::DelegateCache;
use crate::game_actions::Prompt;
use crate::primitives::{
    AbilityId, ActionCount, CardId, GameId, ItemLocation, ManaValue, PlayerId, PointsValue, RaidId,
    RoomId, RoomLocation, Side, TurnNumber,
};
use crate::updates::UpdateTracker;
use crate::with_error::WithError;

/// Mana to be spent only during the `raid_id` raid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificRaidMana {
    pub raid_id: RaidId,
    pub mana: ManaValue,
}

/// Stores a player's mana, both a general-purpose pool and various
/// restricted-purpose pools.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManaState {
    /// General mana, can be used for any purpose.
    pub base_mana: ManaValue,

    /// Mana which can be used only during a specific raid.
    pub specific_raid_mana: Option<SpecificRaidMana>,
}

/// State of a player within a game, containing their score and available
/// resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: PlayerId,
    pub mana_state: ManaState,
    pub actions: ActionCount,
    pub score: PointsValue,

    /// Optionally, an AI Agent for this player. If provided, this agent will be
    /// used to determine game actions instead of prompting for UI input.
    pub agent: Option<AgentData>,

    /// A choice this player is currently facing. Automatically cleared when
    /// a `PromptAction` response is received.
    pub prompt: Option<Prompt>,
}

impl PlayerState {
    /// Create an empty player state.
    pub fn new(id: PlayerId) -> Self {
        Self {
            id,
            agent: None,
            mana_state: ManaState::default(),
            actions: 0,
            score: 0,
            prompt: None,
        }
    }
}

/// Identifies steps within a raid.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, EnumKind)]
#[enum_kind(RaidPhaseKind)]
pub enum RaidPhase {
    /// Raid has been created but does not have a phase yet
    Begin,
    /// The raid has started and the Overlord is deciding whether to activate
    /// the target room.
    Activation,
    /// The defender with the provided ordinal position is currently being
    /// encountered. The Champion is deciding which weapons, if any, to employ.
    ///
    /// Note that defenders are encountered in decreasing position order.
    Encounter(usize),
    /// The Champion has defeated the previous defender and is deciding whether
    /// to continue to encounter the next defender, which has the provided
    /// ordinal position, or retreat.
    ///
    /// Note that defenders are encountered in decreasing position order.
    Continue(usize),
    /// The Champion has bypassed all of the defenders for this room and is now
    /// accessing its contents
    Access,
}

impl RaidPhase {
    pub fn kind(&self) -> RaidPhaseKind {
        self.into()
    }
}

/// Data about an active raid
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    pub room_active: bool,
    /// Cards which have been accessed as part of this raid's Access phase.
    pub accessed: Vec<CardId>,
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

/// Mulligan decision a player made for their opening hand
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum MulliganDecision {
    /// The player has decided to keep their initial hand of 5 cards
    Keep,
    /// The player has elected to draw a new hand of 5 cards
    Mulligan,
}

/// [MulliganDecision]s for both players.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MulliganData {
    /// The mulligan decision for the Overlord player, or None if no decision
    /// has been made.
    pub overlord: Option<MulliganDecision>,
    /// The mulligan decision for the Champion player, or None if no decision
    /// has been made.
    pub champion: Option<MulliganDecision>,
}

impl MulliganData {
    pub fn decision(&self, side: Side) -> Option<&MulliganDecision> {
        match side {
            Side::Overlord => &self.overlord,
            Side::Champion => &self.champion,
        }
        .as_ref()
    }
}

/// Identifies the player whose turn it is
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct TurnData {
    /// Current player whose turn it is
    pub side: Side,
    /// Turn number for that player
    pub turn_number: TurnNumber,
}

/// Describes the final outcome of a game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameOverData {
    /// Player who won the game
    pub winner: Side,
}

/// High level status of a game, including e.g. whose turn it is
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GamePhase {
    ResolveMulligans(MulliganData),
    Play,
    GameOver(GameOverData),
}

/// State and configuration of the overall game, including whose turn it is and
/// whether a raid is active.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameData {
    /// Current [GamePhase].
    pub phase: GamePhase,
    /// Identifies current game turn
    pub turn: TurnData,
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
    /// State for abilities of cards in this game
    pub ability_state: HashMap<AbilityId, AbilityState>,
    /// Next sorting key to use for card moves. Automatically updated by
    /// [Self::next_sorting_key] and [Self::move_card].
    next_sorting_key: u32,
    /// Optional lookup table for delegates present on cards in this game in
    /// order to improve performance
    #[serde(skip)]
    pub delegate_cache: DelegateCache,
}

impl GameState {
    /// Creates a new game with the provided [GameId] and decks for both players
    /// in the [GamePhase::ResolveMulligans] phase.
    ///
    /// Does *not* handle dealing opening hands, prompting for mulligan
    /// decisions, assigning starting mana, etc.
    pub fn new(
        id: GameId,
        overlord_deck: Deck,
        champion_deck: Deck,
        config: GameConfiguration,
    ) -> Self {
        Self {
            id,
            data: GameData {
                phase: GamePhase::ResolveMulligans(MulliganData::default()),
                turn: TurnData { side: Side::Overlord, turn_number: 0 },
                raid: None,
                next_raid_id: 1,
                config,
            },
            overlord_cards: Self::make_deck(&overlord_deck, Side::Overlord),
            champion_cards: Self::make_deck(&champion_deck, Side::Champion),
            overlord: PlayerState::new(overlord_deck.owner_id),
            champion: PlayerState::new(champion_deck.owner_id),
            ability_state: HashMap::new(),
            updates: UpdateTracker::new(!config.simulation),
            next_sorting_key: 1,
            delegate_cache: DelegateCache::default(),
        }
    }

    /// Returns the identity card for the provided Side
    ///
    /// Panics if no identity card is present for this player.
    pub fn identity(&self, side: Side) -> &CardState {
        self.cards(side)
            .iter()
            .find(|c| c.position().kind() == CardPositionKind::Identity)
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
        let card = self.card_mut(card_id);
        card.set_position(key, new_position);
    }

    /// Return a random card in the provided `position`, or None if there are no
    /// cards in that position
    pub fn random_card(&self, position: CardPosition) -> Option<CardId> {
        let mut cards = self.all_cards().filter(|c| c.position() == position);
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
        self.cards(side).iter().filter(move |c| c.position() == position)
    }

    pub fn cards_in_position_mut(
        &mut self,
        side: Side,
        position: CardPosition,
    ) -> impl Iterator<Item = &mut CardState> {
        self.cards_mut(side).iter_mut().filter(move |c| c.position() == position)
    }

    /// Cards owned by a player in a given position, in sorting-key order
    pub fn card_list_for_position(&self, side: Side, position: CardPosition) -> Vec<CardId> {
        let mut result = self.cards_in_position(side, position).collect::<Vec<_>>();
        result.sort();
        result.iter().map(|c| c.id).collect()
    }

    /// Cards in a player's hand, in alphabetical order
    pub fn hand(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position().in_hand())
    }

    /// Cards in a player's deck, in alphabetical order
    pub fn deck(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position().in_deck())
    }

    /// Cards in a player's discard pile, in alphabetical order
    pub fn discard_pile(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position().in_discard_pile())
    }

    /// Returns Overlord cards defending a given room in alphabetical order
    pub fn defenders_alphabetical(&self, room_id: RoomId) -> impl Iterator<Item = &CardState> {
        self.cards_in_position(Side::Overlord, CardPosition::Room(room_id, RoomLocation::Defender))
    }

    /// Overlord cards defending a given room, in sorting-key order (higher
    /// array indices are closer to the front of the room).
    pub fn defender_list(&self, room_id: RoomId) -> Vec<CardId> {
        self.card_list_for_position(
            Side::Overlord,
            CardPosition::Room(room_id, RoomLocation::Defender),
        )
    }

    /// Overlord cards in a given room (not defenders), in alphabetical order
    pub fn occupants(&self, room_id: RoomId) -> impl Iterator<Item = &CardState> {
        self.cards_in_position(Side::Overlord, CardPosition::Room(room_id, RoomLocation::Occupant))
    }

    /// All Overlord cards located within a given room, defenders and occupants,
    /// in alphabetical order.
    pub fn defenders_and_occupants(&self, room_id: RoomId) -> impl Iterator<Item = &CardState> {
        self.cards(Side::Overlord)
            .iter()
            .filter(move |c| matches!(c.position(), CardPosition::Room(r, _) if r == room_id))
    }

    /// All overlord defenders in play, whether face-up or face-down.
    pub fn minions(&self) -> impl Iterator<Item = &CardState> {
        self.cards(Side::Overlord)
            .iter()
            .filter(move |c| matches!(c.position(), CardPosition::Room(_, RoomLocation::Defender)))
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

    /// All Card IDs present in this game.
    ///
    /// Overlord cards in alphabetical order followed by Champion cards in
    /// alphabetical order.
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

    /// Helper method to return the current [RaidData] or an error when one is
    /// expected to exist.
    pub fn raid(&self) -> Result<&RaidData> {
        self.data.raid.as_ref().with_error(|| "Expected Raid")
    }

    /// Mutable version of [Self::raid].
    pub fn raid_mut(&mut self) -> Result<&mut RaidData> {
        self.data.raid.as_mut().with_error(|| "Expected Raid")
    }

    /// Retrieves the [AbilityState] for an [AbilityId]
    pub fn ability_state(&self, ability_id: impl Into<AbilityId>) -> Option<&AbilityState> {
        self.ability_state.get(&ability_id.into())
    }

    /// Returns a mutable [AbilityState] for an [AbilityId], creating a new one
    /// if one has not previously been set
    pub fn ability_state_mut(&mut self, ability_id: impl Into<AbilityId>) -> &mut AbilityState {
        self.ability_state.entry(ability_id.into()).or_insert_with(AbilityState::default)
    }

    /// Create card states for a deck
    fn make_deck(deck: &Deck, side: Side) -> Vec<CardState> {
        let mut result =
            vec![CardState::new(CardId::new(side, 0), deck.identity, true /* is_identity */)];

        result.extend(deck.card_names().iter().enumerate().map(move |(index, name)| {
            CardState::new(CardId::new(side, index + 1), *name, false /* is_identity */)
        }));

        result
    }
}
