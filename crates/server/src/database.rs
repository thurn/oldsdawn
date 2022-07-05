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

use anyhow::Result;
use data::game::GameState;
use data::player_data::PlayerData;
use data::player_name::PlayerId;
use data::primitives::GameId;
use data::with_error::WithError;
use once_cell::sync::Lazy;
use prost::Message;
use protos::spelldawn::player_identifier::PlayerIdentifierType;
use protos::spelldawn::PlayerIdentifier;
use rules::dispatch;
use serde_json::{de, ser};
use sled::{Db, Tree};

static DATABASE_PATH: Lazy<Mutex<Option<String>>> = Lazy::new(|| Mutex::new(None));

static DATABASE: Lazy<Db> = Lazy::new(|| {
    let path = DATABASE_PATH.lock().expect("path lock").clone();
    sled::open(path.unwrap_or_else(|| "db".to_string())).expect("Unable to open database")
});

/// Overrides the path used for the database, e.g. in order to use
/// Application.persistentDataPath in Unity. Must be called before any database
/// access in order to have effect.
pub fn override_path(path: String) {
    DATABASE_PATH.lock().expect("path lock").replace(path);
}

/// Abstraction layer for interacting with the database
pub trait Database: Send + Sync {
    /// Generate a new unique [GameId] to be used for a new game
    fn generate_game_id(&self) -> Result<GameId>;

    /// Check whether a given game exists.
    fn has_game(&self, id: GameId) -> Result<bool>;

    /// Look up an ongoing [GameState] by ID. It is an error to look up an ID
    /// which does not exist.
    fn game(&self, id: GameId) -> Result<GameState>;

    /// Store a [GameState] in the database based on its ID.
    fn write_game(&mut self, game: &GameState) -> Result<()>;

    /// Retrieve a player's [PlayerData], if this player exists.
    ///
    /// It is always an error to invoke this method with a `PlayerId::Named`
    /// identifier.
    fn player(&self, player_id: PlayerId) -> Result<Option<PlayerData>>;

    /// Store a [PlayerData] in the database based on its ID.
    fn write_player(&mut self, player: &PlayerData) -> Result<()>;

    /// Convert a [PlayerIdentifier] to a [PlayerId], either by looking up its
    /// existing mapping or storing a new randomly-generated ID for this
    /// identifier.
    fn adapt_player_identifier(&mut self, identifier: &PlayerIdentifier) -> Result<PlayerId>;
}

/// Database implementation based on the sled database
pub struct SledDatabase {
    /// Whether to flush after each write() call. This is needed for the unity
    /// plugin because auto-flush doesn't work on devices.
    pub flush_on_write: bool,
}

impl Database for SledDatabase {
    // Longer term, I've been thinking it might make sense to use different storage
    // layers for different things, e.g. an embedded database for game state vs a
    // cloud storage solution for collection management.

    fn generate_game_id(&self) -> Result<GameId> {
        Ok(GameId::new(DATABASE.generate_id().with_error(|| "Error generating ID")?))
    }

    fn has_game(&self, id: GameId) -> Result<bool> {
        games()?.contains_key(id.key()).with_error(|| format!("Error reading key {:?}", id))
    }

    fn game(&self, id: GameId) -> Result<GameState> {
        let content = games()?
            .get(id.key())
            .with_error(|| format!("Error reading  game: {:?}", id))?
            .with_error(|| format!("Game not found: {:?}", id))?;
        let res: std::result::Result<GameState, serde_json::Error> =
            de::from_slice(content.as_ref());
        if let Err(e) = res {
            panic!("ERROR: {:?}", e);
        }
        let mut game = de::from_slice(content.as_ref())
            .with_error(|| format!("Error deserializing game {:?}", id))?;
        dispatch::populate_delegate_cache(&mut game);
        Ok(game)
    }

    fn write_game(&mut self, game: &GameState) -> Result<()> {
        let serialized =
            ser::to_vec(game).with_error(|| format!("Error serializing game {:?}", game.id))?;
        let result = games()?
            .insert(game.id.key(), serialized)
            .map(|_| ()) // Ignore previously-set value
            .with_error(|| format!("Error writing game {:?}", game.id));

        if self.flush_on_write {
            DATABASE.flush()?;
        }

        result
    }

    fn player(&self, player_id: PlayerId) -> Result<Option<PlayerData>> {
        Ok(
            if let Some(content) = players()?
                .get(player_id.database_key()?)
                .with_error(|| format!("Error reading player: {:?}", player_id))?
            {
                de::from_slice(content.as_ref())
                    .with_error(|| format!("Error deserializing player {:?}", player_id))?
            } else {
                None
            },
        )
    }

    fn write_player(&mut self, player: &PlayerData) -> Result<()> {
        let serialized = ser::to_vec(player)
            .with_error(|| format!("Error serializing player {:?}", player.id))?;
        let result = players()?
            .insert(player.id.database_key()?, serialized)
            .map(|_| ()) // Ignore previously-set value
            .with_error(|| format!("Error writing player {:?}", player.id));

        if self.flush_on_write {
            DATABASE.flush()?;
        }

        result
    }

    fn adapt_player_identifier(&mut self, identifier: &PlayerIdentifier) -> Result<PlayerId> {
        if let Some(PlayerIdentifierType::ServerIdentifier(bytes)) =
            &identifier.player_identifier_type
        {
            return adapters::named_player_id(bytes);
        }

        let serialized = identifier.encode_to_vec();
        let ids = player_ids()?;
        if let Some(key) = ids.get(&serialized).with_error(|| "Error reading player ID")? {
            Ok(PlayerId::Database(u64::from_be_bytes(key.as_ref().try_into()?)))
        } else {
            let result = DATABASE.generate_id().with_error(|| "Error generating ID")?;
            ids.insert(&serialized, &result.to_be_bytes()).with_error(|| "Error inserting ID")?;
            Ok(PlayerId::Database(result))
        }
    }
}

fn games() -> Result<Tree> {
    DATABASE.open_tree("games").with_error(|| "Error opening the 'games' table")
}

fn players() -> Result<Tree> {
    DATABASE.open_tree("players").with_error(|| "Error opening the 'players' table")
}

fn player_ids() -> Result<Tree> {
    DATABASE.open_tree("player_ids").with_error(|| "Error opening the 'player_ids' table")
}
