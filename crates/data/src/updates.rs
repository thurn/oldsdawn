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

//! Used to track mutations during a game for rendering by the client

#![allow(clippy::use_self)] // Required to use EnumKind

use enum_kinds::EnumKind;
use serde::{Deserialize, Serialize};

use crate::primitives::{CardId, PointsValue, RoomId, Side};

/// Identifies the source or target of a game interaction
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum InteractionObjectId {
    CardId(CardId),
    Identity(Side),
    Deck(Side),
    DiscardPile(Side),
}

/// Indicates one card targeted another with an effect.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct TargetedInteraction {
    /// Source of the effect
    pub source: InteractionObjectId,
    /// Target of the effect
    pub target: InteractionObjectId,
}

/// Represents an update to the state of the game which should be translated
/// into a client update
#[derive(PartialEq, Eq, Hash, Debug, Clone, EnumKind)]
#[enum_kind(GameUpdateKind, derive(Ord, PartialOrd))]
pub enum GameUpdate {
    /// Indicates that a new hand of cards has been drawn for the provided
    /// player.
    DrawHand(Side),
    /// A user kept a hand with the provided card list during the 'resolve
    /// mulligans' step
    KeepHand(Side, Vec<CardId>),
    /// A user mulliganed a hand with the first provided card ID list and
    /// received a hand with the second provided card ID list during the
    /// 'resolve mulligans' step.
    MulliganHand(Side, Vec<CardId>, Vec<CardId>),
    /// A room has been leveled up
    LevelUpRoom(RoomId),
    /// A raid has started on the indicated room
    InitiateRaid(RoomId),
    /// A card has moved from a deck to a player's hand.
    DrawCard(CardId),
    /// A card has been shuffled back into a player's deck
    ShuffleIntoDeck(CardId),
    /// A card has been completely removed from the game
    DestroyCard(CardId),
    /// Indicates that one card or game object targeted another with an effect.
    TargetedInteraction(TargetedInteraction),
    /// A card has been scored by the overlord player
    OverlordScoreCard(CardId, PointsValue),
    /// A card has been scored by the champion player
    ChampionScoreCard(CardId, PointsValue),

    GeneralUpdate,

    /// A card has become revealed to the opponent. If this occurs while a card
    /// is changing zones, this update should be added before `MoveCard` to
    /// move the card to its final destination.
    RevealToOpponent(CardId),
    /// The current raid has gained access to the indicated room
    RaidAccess(RoomId),
    /// The game has ended and the indicated player has won
    GameOver(Side),
    MoveToZone(CardId),

    /// Indicates that a player's turn has started
    StartTurn(Side),
}

impl GameUpdate {
    pub fn kind(&self) -> GameUpdateKind {
        self.into()
    }
}

/// Tracks game mutations for a given network request. If a vector is present
/// here, then code which mutates the GameState is also responsible for
/// appending a [GameUpdate] which describes the mutation. If no vector is
/// present it means update tracking is currently disabled (e.g. because we are
/// running in simulation mode).
#[derive(Debug, Clone, Default)]
pub struct UpdateTracker {
    update_list: Option<Vec<GameUpdate>>,
}

impl UpdateTracker {
    pub fn new(enabled: bool) -> Self {
        Self { update_list: enabled.then(|| vec![GameUpdate::GeneralUpdate]) }
    }

    pub fn list(&self) -> Option<&Vec<GameUpdate>> {
        self.update_list.as_ref()
    }

    /// Appends a [GameUpdate] to the update list.
    pub fn push(&mut self, update: GameUpdate) {
        if let Some(vec) = &mut self.update_list {
            vec.push(update)
        }
    }
}
