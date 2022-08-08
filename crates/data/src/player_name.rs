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
use clap::ArgEnum;
use serde::{Deserialize, Serialize};
use with_error::fail;

/// Identifies a player across different games
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum PlayerId {
    /// ID stored in the database, i.e. a human player
    Database(u64),
    /// Known player, i.e. an AI agent
    Named(NamedPlayer),
}

impl PlayerId {
    /// Returns the database key for this player, or an error if this is not a
    /// database-backed player ID.
    pub fn database_key(&self) -> Result<[u8; 8]> {
        match self {
            PlayerId::Database(key) => Ok(key.to_be_bytes()),
            _ => fail!("Expected PlayerId::Database"),
        }
    }
}

/// Identifies a named AI player
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize, ArgEnum)]
pub enum NamedPlayer {
    /// Uses canonical decklists, does not take any actions
    TestNoAction,
    TestAlphaBeta,
}
