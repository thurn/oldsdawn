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

use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Result;
use cards::decklists;
use dashmap::DashMap;
use data::deck::Deck;
use data::game::GameState;
use data::primitives::{GameId, PlayerId, Side};
use data::with_error::WithError;
use once_cell::sync::Lazy;

use crate::database::Database;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);
static DATABASE: Lazy<DashMap<GameId, GameState>> = Lazy::new(DashMap::new);

pub struct InMemoryDatabase;

impl Database for InMemoryDatabase {
    fn generate_game_id(&self) -> Result<GameId> {
        Ok(GameId::new(NEXT_ID.fetch_add(1, Ordering::Relaxed)))
    }

    fn has_game(&self, id: GameId) -> Result<bool> {
        Ok(DATABASE.contains_key(&id))
    }

    fn game(&self, id: GameId) -> Result<GameState> {
        DATABASE.get(&id).map(|g| g.clone()).with_error(|| format!("Game not found: {:?}", id))
    }

    fn write_game(&mut self, game: &GameState) -> Result<()> {
        DATABASE.insert(game.id, game.clone());
        Ok(())
    }

    fn deck(&self, player_id: PlayerId, side: Side) -> Result<Deck> {
        Ok(decklists::canonical_deck(player_id, side))
    }
}
