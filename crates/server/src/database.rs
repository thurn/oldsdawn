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

use anyhow::{Context, Result};
use bincode;
use data::card_name::CardName;
use data::deck::Deck;
use data::game::GameState;
use data::primitives::{GameId, PlayerId, Side};
use data::with_error::WithError;
use maplit::hashmap;
use once_cell::sync::Lazy;
use sled::{Db, Tree};

static DATABASE: Lazy<Db> = Lazy::new(|| sled::open("db").expect("Unable to open database"));

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
pub struct SledDatabase;

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
        bincode::deserialize(content.as_ref())
            .with_error(|| format!("Error deserializing game {:?}", id))
    }

    fn write_game(&mut self, game: &GameState) -> Result<()> {
        let serialized = bincode::serialize(game)
            .with_error(|| format!("Error serializing game {:?}", game.id))?;
        games()?
            .insert(game.id.key(), serialized)
            .map(|_| ()) // Ignore previously-set value
            .with_error(|| format!("Error writing game {:?}", game.id))
    }

    fn deck(&self, player_id: PlayerId, side: Side) -> Result<Deck> {
        Ok(if side == Side::Champion {
            Deck {
                owner_id: player_id,
                identity: CardName::TestChampionIdentity,
                cards: hashmap! {
                    CardName::Lodestone => 15,
                    CardName::Greataxe => 15,
                    CardName::ArcaneRecovery => 15,
                },
            }
        } else {
            Deck {
                owner_id: player_id,
                identity: CardName::TestOverlordIdentity,
                cards: hashmap! {
                    CardName::DungeonAnnex => 15,
                    CardName::IceDragon => 15,
                    CardName::GoldMine => 15
                },
            }
        })
    }
}

fn games() -> Result<Tree> {
    DATABASE.open_tree("games").with_error(|| "Error opening the 'games table")
}
