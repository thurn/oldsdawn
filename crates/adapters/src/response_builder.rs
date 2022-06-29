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

use data::primitives::Side;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{
    CardIdentifier, GameView, ObjectPosition, PlayerName, UpdateGameViewCommand,
};

pub struct ResponseState {
    pub animate: bool,
    pub is_final_update: bool,
}

pub struct ResponseBuilder {
    pub user_side: Side,
    pub state: ResponseState,
    pub commands: Vec<Command>,

    /// Tracks the positions of client cards as of the most recently-seen
    /// snapshot. Can be used to customize animation behavior.
    pub last_snapshot_positions: HashMap<CardIdentifier, ObjectPosition>,
}

impl ResponseBuilder {
    pub fn new(user_side: Side, state: ResponseState) -> Self {
        Self { user_side, state, commands: vec![], last_snapshot_positions: HashMap::default() }
    }

    pub fn push(&mut self, command: Command) {
        self.commands.push(command);
    }

    pub fn push_game_view(&mut self, game: GameView) {
        for card in &game.cards {
            if let (Some(id), Some(position)) = (card.card_id, card.card_position.clone()) {
                self.last_snapshot_positions.insert(id, position);
            }
        }

        self.commands.push(Command::UpdateGameView(UpdateGameViewCommand {
            game: Some(game),
            animate: self.state.animate,
        }));
    }

    pub fn to_player_name(&self, side: Side) -> i32 {
        if side == self.user_side {
            PlayerName::User as i32
        } else {
            PlayerName::Opponent as i32
        }
    }
}
