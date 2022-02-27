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

use data::deck::Deck;
use data::game::GameState;
use data::primitives::{GameId, PlayerId, Side};
use server::database::Database;

#[derive(Clone, Debug, Default)]
pub struct FakeDatabase {
    pub generated_game_id: Option<GameId>,
    pub game: Option<GameState>,
    pub overlord_deck: Option<Deck>,
    pub champion_deck: Option<Deck>,
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
    fn generate_game_id(&self) -> anyhow::Result<GameId> {
        Ok(self.generated_game_id.expect("generated_game_id"))
    }

    fn has_game(&self, id: GameId) -> anyhow::Result<bool> {
        Ok(matches!(&self.game, Some(game) if game.id == id))
    }

    fn game(&self, _id: GameId) -> anyhow::Result<GameState> {
        Ok(self.game.clone().expect("game"))
    }

    fn write_game(&mut self, game: &GameState) -> anyhow::Result<()> {
        self.game = Some(game.clone());
        Ok(())
    }

    fn deck(&self, _player_id: PlayerId, side: Side) -> anyhow::Result<Deck> {
        Ok(match side {
            Side::Overlord => self.overlord_deck.clone().expect("overlord_deck"),
            Side::Champion => self.champion_deck.clone().expect("champion_deck"),
        })
    }
}
