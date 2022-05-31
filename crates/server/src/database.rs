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

//! Core database implementation, handles querying and storing game state.

use std::sync::Mutex;

use anyhow::{Context, Result};
use bincode;
use cards::decklists;
use data::deck::Deck;
use data::game::GameState;
use data::primitives::{GameId, PlayerId, Side};
use data::with_error::WithError;
use once_cell::sync::Lazy;
use rules::dispatch;
use sled::{Db, Tree};

static DATABASE_PATH: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

static DATABASE: Lazy<Db> = Lazy::new(|| {
    let path = DATABASE_PATH.lock().unwrap().clone();
    sled::open(path.unwrap_or_else(|| "db".to_string())).expect("Unable to open database")
});

/// Overrides the path used for the database, e.g. in order to use
/// Application.persistentDataPath in Unity. Must be called before any database
/// access in order to have effect.
pub fn override_path(path: String) {
    DATABASE_PATH.lock().unwrap().replace(path);
}

/// Abstraction layer for interacting with the database
pub trait Database {
    /// Generate a new unique [GameId] to be used for a new game
    fn generate_game_id(&self) -> Result<GameId>;
    /// Check whether a given game exists.
    fn has_game(&self, id: GameId) -> Result<bool>;
    /// Look up an ongoing [GameState] by ID. It is an error to look up an ID
    /// which does not exist.
    fn game(&self, id: GameId) -> Result<GameState>;
    /// Store a [GameState] in the database based on its ID.
    fn write_game(&mut self, game: &GameState) -> Result<()>;
    /// Retrieve a player's [Deck] for a given [Side].
    fn deck(&self, player_id: PlayerId, side: Side) -> Result<Deck>;
}

/// Database implementation based on the sled database
pub struct SledDatabase {
    /// Whether to flush after each write() call. This is needed for the unity
    /// plugin because auto-flush doesn't work on devices.
    pub flush_on_write: bool,
}

impl Database for SledDatabase {
    fn generate_game_id(&self) -> Result<GameId> {
        Ok(GameId::new(DATABASE.generate_id().with_context(|| "Error generating ID")?))
    }

    fn has_game(&self, id: GameId) -> Result<bool> {
        games()?.contains_key(id.key()).with_error(|| format!("Error reading key {:?}", id))
    }

    fn game(&self, id: GameId) -> Result<GameState> {
        let content = games()?
            .get(id.key())
            .with_error(|| format!("Error reading  game: {:?}", id))?
            .with_error(|| format!("Game not found: {:?}", id))?;
        let mut game = bincode::deserialize(content.as_ref())
            .with_error(|| format!("Error deserializing game {:?}", id))?;
        dispatch::populate_delegate_cache(&mut game);
        Ok(game)
    }

    fn write_game(&mut self, game: &GameState) -> Result<()> {
        let serialized = bincode::serialize(game)
            .with_error(|| format!("Error serializing game {:?}", game.id))?;
        let result = games()?
            .insert(game.id.key(), serialized)
            .map(|_| ()) // Ignore previously-set value
            .with_error(|| format!("Error writing game {:?}", game.id));

        if self.flush_on_write {
            DATABASE.flush()?;
        }

        result
    }

    fn deck(&self, player_id: PlayerId, side: Side) -> Result<Deck> {
        Ok(if side == Side::Champion {
            Deck { owner_id: player_id, ..decklists::CANONICAL_CHAMPION.clone() }
        } else {
            Deck { owner_id: player_id, ..decklists::CANONICAL_OVERLORD.clone() }
        })
    }
}

fn games() -> Result<Tree> {
    DATABASE.open_tree("games").with_error(|| "Error opening the 'games' table")
}
