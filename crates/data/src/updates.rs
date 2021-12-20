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

use crate::primitives::{CardId, PointsValue, RoomId, Side};
use serde::{Deserialize, Serialize};

/// Identifies the source or target of a game interaction
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum InteractionObjectId {
    CardId(CardId),
    Identity(Side),
    Deck(Side),
    Hand(Side),
    DiscardPile(Side),
}

/// Indicates one card targeted another with an effect.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub struct TargetedInteraction {
    /// Source of the effect
    pub source: InteractionObjectId,
    /// Target of the effect
    pub target: InteractionObjectId,
    /// If true, the target will be removed from the raid display and returned to its original game
    /// position
    pub remove_from_raid: bool,
}

/// Represents an update to the state of the game which should be translated into a client update
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum GameUpdate {
    /// Indicates a general game state change, such as modifying a player's mana or the current
    /// turn number.
    UpdateGame,
    /// Indicates a general card state change, such as a modification to its attack value.
    UpdateCard(CardId),
    /// Indicates that a player's opening hand has been drawn and may be kept or mulliganed
    ShowOpeningHand(Side),
    /// Indicates that a player has kept their current hand
    KeepOpeningHand(Side),
    /// Indicates that a player's turn has started
    StartTurn(Side),
    /// A card has moved from a player's deck to that player's hand.
    DrawCard(CardId),
    /// A card has been removed from the game or shuffled back into a player's deck
    DestroyCard(CardId),
    /// A card has been moved to a new game location
    MoveCard(CardId),
    /// A room has been leveled up
    LevelUpRoom(RoomId),
    /// A raid has started on the indicated room
    InitiateRaid(RoomId),
    /// Indicates that one card or game object targeted another with an effect.
    TargetedInteraction(TargetedInteraction),
    /// The current raid has gained access to the indicated room
    RaidAccess(RoomId),
    /// A card has been scored by the indicated player
    ScoreCard(Side, CardId, PointsValue),
    /// A raid has ended
    EndRaid,
    /// The game has ended and the indicated player has won
    GameOver(Side),
}

/// Tracks game mutations for a given network request. If a vector is present here, then code which
/// mutates the GameState is also responsible for appending a [GameUpdate] which describes the
/// mutation. If no vector is present it means update tracking is currently disabled (e.g. because
/// we are running in simulation mode).
#[derive(Debug, Clone, Default)]
pub struct UpdateTracker {
    pub update_list: Option<Vec<GameUpdate>>,
}

impl UpdateTracker {
    pub fn push(&mut self, update: GameUpdate) {
        if let Some(vec) = &mut self.update_list {
            vec.push(update)
        }
    }
}
