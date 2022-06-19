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
use crate::primitives::{AbilityId, CardId, Side};

#[derive(PartialEq, Eq, Hash, Debug, Clone, Copy)]
pub enum StackId {
    CardId(CardId),
    AbilityId(AbilityId),
}

impl From<CardId> for StackId {
    fn from(card_id: CardId) -> Self {
        Self::CardId(card_id)
    }
}

impl From<AbilityId> for StackId {
    fn from(ability_id: AbilityId) -> Self {
        Self::AbilityId(ability_id)
    }
}

#[derive(Debug, Clone)]
pub enum GameUpdate {
    /// One or more cards have been drawn by the [Side] player.
    DrawCards(Side, Vec<CardId>),
}

#[derive(Debug, Clone)]
pub enum UpdateStep {
    GameUpdate(GameUpdate),
    Sync(Box<GameState>),
}

#[derive(Debug, Clone, Default)]
pub struct UpdateTracker {
    enabled: bool,
    updates: Vec<UpdateStep>,
}

impl UpdateTracker {
    pub fn new(enabled: bool) -> Self {
        Self { enabled, updates: vec![] }
    }

    pub fn updates(&self) -> impl Iterator<Item = &UpdateStep> {
        self.updates.iter()
    }

    pub fn push(&mut self, game: GameState, update: GameUpdate) {
        if self.enabled {
            self.updates.push(UpdateStep::Sync(Box::new(game)));
            self.updates.push(UpdateStep::GameUpdate(update));
        }
    }
}
