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

use crate::game::GameState;
use crate::primitives::{CardId, GameObjectId, Side};

/// Indicates one game object targeted another with an effect.
///
/// Typically represented in animation as a projectile being fired.
#[derive(Debug, Clone)]
pub struct TargetedInteraction {
    pub source: GameObjectId,
    pub target: GameObjectId,
}

/// Represents a change to the state of the game which should be translated
/// into a client animation
#[derive(Debug, Clone)]
pub enum GameUpdate {
    /// One or more cards have been drawn by the [Side] player.
    DrawCards(Side, Vec<CardId>),
    /// A project card has been turned face-up.
    UnveilProject(CardId),
    /// A minion card has been turned face-up.
    SummonMinion(CardId),
    /// Cards have been accessed during a raid
    CardsAccessed(Vec<CardId>),
    /// See [TargetedInteraction].
    TargetedInteraction(TargetedInteraction),
}

/// A step in the animation process
#[derive(Debug, Clone)]
pub struct UpdateStep {
    pub snapshot: GameState,
    pub update: GameUpdate,
}

/// Standard enum used by APIs to configure their update tracking behavior.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Updates {
    /// Game updates should not be tracked by the receiver
    Ignore,
    /// Game updates should be tracked by the receiver
    Push,
}

/// Tracks game mutations for a game action.
///
/// Some game state changes in Spelldawn require custom animations in the UI in
/// order to communicate their effects clearly. In order to implement the
/// animation system, code which mutates game state can also call
/// [GameState::push_update] and provide a [GameUpdate] to record the action
/// they took. The way this process works is that a snapshot of the game state
/// is stored (to capture any mutations that occurred *before* the animation),
/// and then the update is stored. During the animation process, the
/// stored snapshots and [GameUpdate]s are played back sequentially.
///
/// Many types of state changes are handled automatically by the game state
/// snapshot system, so appending an update is only needed for custom
/// animations. For example the system will correctly detect and animate a card
/// which has moved to a new position.
#[derive(Debug, Clone)]
pub struct UpdateTracker {
    /// Used to globally disable or enable update tracking
    pub state: Updates,
    /// List of update steps, either full snapshots of the game state or
    /// individual mutations.
    pub steps: Vec<UpdateStep>,
}

impl Default for UpdateTracker {
    fn default() -> Self {
        Self { state: Updates::Ignore, steps: vec![] }
    }
}

impl UpdateTracker {
    pub fn new(updates: Updates) -> Self {
        Self { state: updates, steps: vec![] }
    }
}
