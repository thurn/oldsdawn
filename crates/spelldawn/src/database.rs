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

pub fn generate_id() -> Result<u64> {
    DATABASE.generate_id().with_context(|| "Error generating ID")
}

pub fn games() -> Result<Tree> {
    DATABASE.open_tree("games").with_context(|| "Error opening the 'games table")
}

pub fn game(id: GameId) -> Result<GameState> {
    let content = games()?
        .get(id.key())
        .with_context(|| format!("Error reading  game: {:?}", id))?
        .with_context(|| format!("Game not found: {:?}", id))?;
    bincode::deserialize(content.as_ref())
        .with_context(|| format!("Error deserializing game {:?}", id))
}

pub fn write_game(game: &GameState) -> Result<()> {
    let serialized = bincode::serialize(game)
        .with_context(|| format!("Error serializing game {:?}", game.id()))?;
    games()?
        .insert(game.id().key(), serialized)
        .map(|_| ())
        .with_context(|| format!("Error writing game {:?}", game.id()))
}
