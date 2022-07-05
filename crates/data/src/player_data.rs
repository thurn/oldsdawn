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

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::card_name::CardName;
use crate::deck::Deck;
use crate::player_name::PlayerId;
use crate::primitives::{DeckId, GameId};

/// Data for a player's request to create a new game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewGameRequest {
    pub deck_id: DeckId,
}

/// Represents the state of a game the player is participating in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CurrentGame {
    /// The player has initiated a request to create a game
    Requested(NewGameRequest),
    /// The player is currently playing in the [GameId] game.
    Playing(GameId),
}

/// Represents a player's stored data.
///
/// For a player's state *within a given game* see `PlayerState`.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerData {
    /// Unique identifier for this player
    pub id: PlayerId,
    /// Game this player is currently participating in, if any.
    pub current_game: Option<CurrentGame>,
    /// This player's saved decks.
    pub decks: Vec<Deck>,
    /// Cards owned by this player
    #[serde_as(as = "Vec<(_, _)>")]
    pub collection: HashMap<CardName, u32>,
}

impl PlayerData {
    /// Returns the [DeckId] this player requested to use for a new game.
    pub fn requested_deck_id(&self) -> Option<DeckId> {
        match &self.current_game {
            Some(CurrentGame::Requested(request)) => Some(request.deck_id),
            _ => None,
        }
    }

    /// Retrieves one of a player's decks based on its [DeckId].
    pub fn deck(&self, deck_id: DeckId) -> &Deck {
        &self.decks[deck_id.value as usize]
    }

    pub fn deck_mut(&mut self, deck_id: DeckId) -> &mut Deck {
        &mut self.decks[deck_id.value as usize]
    }
}

/// Returns the [GameId] an optional [PlayerData] is currently playing in, if
/// any.
pub fn current_game_id(data: Option<PlayerData>) -> Option<GameId> {
    match data.as_ref().and_then(|player| player.current_game.as_ref()) {
        Some(CurrentGame::Playing(id)) => Some(*id),
        _ => None,
    }
}
