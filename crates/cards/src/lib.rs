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

//! Contains the definitions for all cards in the game.

pub mod minions;
pub mod projects;
pub mod schemes;
pub mod spells;
pub mod test_cards;
pub mod weapons;

/// Initializes cards and returns the number of discovered cards.
///
/// In order for `linkme` to find the card definitions we need to call a
/// function on each module, not completely sure if this is expected behavior or
/// a bug.
pub fn initialize() -> usize {
    minions::initialize();
    projects::initialize();
    schemes::initialize();
    spells::initialize();
    test_cards::initialize();
    weapons::initialize();

    rules::CARDS.len()
}
