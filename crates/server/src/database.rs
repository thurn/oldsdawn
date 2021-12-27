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

use anyhow::{Context, Result};
use bincode;
use data::game::GameState;
use data::primitives::GameId;
use once_cell::sync::Lazy;
use sled::{Db, Tree};

static DATABASE: Lazy<Db> = Lazy::new(|| sled::open("db").expect("Unable to open database"));

/// Abstraction layer for interacting with the database
pub trait Database {
    /// Generate a new unique [GameId] to be used for a new game
    fn generate_game_id(&self) -> Result<GameId>;
    /// Look up an ongoing [GameState] by ID. It is an error to look up an ID
    /// which does not exist.
    fn game(&self, id: GameId) -> Result<GameState>;
    /// Store a [GameState] in the database based on its ID.
    fn write_game(&mut self, game: &GameState) -> Result<()>;
}

pub struct SledDatabase;

impl Database for SledDatabase {
    fn generate_game_id(&self) -> Result<GameId> {
        Ok(GameId::new(DATABASE.generate_id().with_context(|| "Error generating ID")?))
    }

    fn game(&self, id: GameId) -> Result<GameState> {
        let content = games()?
            .get(id.key())
            .with_context(|| format!("Error reading  game: {:?}", id))?
            .with_context(|| format!("Game not found: {:?}", id))?;
        bincode::deserialize(content.as_ref())
            .with_context(|| format!("Error deserializing game {:?}", id))
    }

    fn write_game(&mut self, game: &GameState) -> Result<()> {
        let serialized = bincode::serialize(game)
            .with_context(|| format!("Error serializing game {:?}", game.id))?;
        games()?
            .insert(game.id.key(), serialized)
            .map(|_| ()) // Ignore previously-set value
            .with_context(|| format!("Error writing game {:?}", game.id))
    }
}

fn games() -> Result<Tree> {
    DATABASE.open_tree("games").with_context(|| "Error opening the 'games table")
}
