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

use anyhow::Result;
use data::game::GameState;
use data::player_data::PlayerData;
use data::player_name::PlayerId;
use data::primitives::GameId;
use protos::spelldawn::player_identifier::PlayerIdentifierType;
use protos::spelldawn::PlayerIdentifier;
use server::database::Database;

#[derive(Clone, Debug, Default)]
pub struct FakeDatabase {
    pub generated_game_id: Option<GameId>,
    pub game: Option<GameState>,
    pub players: HashMap<PlayerId, PlayerData>,
}

impl FakeDatabase {
    pub fn game(&self) -> &GameState {
        self.game.as_ref().expect("game")
    }

    pub fn game_mut(&mut self) -> &mut GameState {
        self.game.as_mut().expect("game")
    }
}

impl Database for FakeDatabase {
    fn generate_game_id(&self) -> Result<GameId> {
        Ok(self.generated_game_id.expect("generated_game_id"))
    }

    fn has_game(&self, id: GameId) -> Result<bool> {
        Ok(matches!(&self.game, Some(game) if game.id == id))
    }

    fn game(&self, _id: GameId) -> Result<GameState> {
        Ok(self.game.clone().expect("game"))
    }

    fn write_game(&mut self, game: &GameState) -> Result<()> {
        self.game = Some(game.clone());
        Ok(())
    }

    fn player(&self, player_id: PlayerId) -> Result<Option<PlayerData>> {
        Ok(Some(self.players[&player_id].clone()))
    }

    fn write_player(&mut self, player: &PlayerData) -> Result<()> {
        self.players.insert(player.id, player.clone());
        Ok(())
    }

    fn adapt_player_identifier(&mut self, identifier: &PlayerIdentifier) -> Result<PlayerId> {
        match identifier.player_identifier_type.clone().unwrap() {
            PlayerIdentifierType::ServerIdentifier(bytes) => {
                Ok(PlayerId::Database(u64::from_be_bytes(bytes.try_into().unwrap())))
            }
            _ => panic!("Unsupported identifier type"),
        }
    }
}

pub fn to_player_identifier(id: PlayerId) -> PlayerIdentifier {
    let value = match id {
        PlayerId::Database(value) => value,
        _ => panic!("Unsupported PlayerId type"),
    };

    PlayerIdentifier {
        player_identifier_type: Some(PlayerIdentifierType::ServerIdentifier(
            value.to_be_bytes().to_vec(),
        )),
    }
}
