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

use anyhow::Result;

/// A generic game state used by an AI algorithm.
///
/// Keeping the AI search algorithm implementation generic when possible is
/// useful for testing. We use a much simpler game with a known-optimal
/// strategy (the game of Nim) to sanity-check that the AI implementations are
/// doing broadly correct things.
pub trait GameStateNode {
    /// A player in the game.
    type PlayerName: Eq + Copy;

    /// A game action to transition the game to a new state.
    type Action: Copy;

    /// Create a copy of this search node to be mutated by selection algorithms.
    /// A basic implementation of this would be to simply call `.clone()`, but
    /// sometimes parts of the game state are only for display and are not
    /// relevant for selection algorithms.
    fn make_copy(&self) -> Self;

    /// Returns player whose turn it is currently, or `None` if the game has
    /// ended.
    fn current_turn(&self) -> Option<Self::PlayerName>;

    /// Returns an iterator over actions that the current player can legally
    /// take in the current game state.
    fn legal_actions<'a>(&'a self) -> Result<Box<dyn Iterator<Item = Self::Action> + 'a>>;

    /// Apply the result of a given action to this game state.
    fn execute_action(&mut self, player: Self::PlayerName, action: Self::Action) -> Result<()>;
}
