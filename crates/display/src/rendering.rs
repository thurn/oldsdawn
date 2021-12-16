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

use model::game::GameState;
use model::primitives::Side;
use protos::spelldawn::{GameId, GameView, PlayerView};

pub fn basic_game_view(game: &GameState, viewer_side: Side) -> Option<GameView> {
    if game.modified() {
        Some(GameView {
            game_id: Some(game_id(game.id())),
            user: None,
            opponent: None,
            arena: None,
            current_priority: 0,
        })
    } else {
        None
    }
}

pub fn game_id(string: &str) -> GameId {
    GameId { value: string.to_string() }
}

pub fn basic_player_view(game: &GameState, side: Side) -> PlayerView {
    PlayerView {
        player_info: None,
        score: None,
        hand: None,
        mana: None,
        discard_pile: None,
        action_tracker: None,
        deck: None,
    }
}
