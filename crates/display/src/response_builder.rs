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

use data::primitives::Side;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::PlayerName;

pub struct ResponseBuilder {
    pub user_side: Side,
    pub animate: bool,
    pub commands: Vec<Command>,
}

impl ResponseBuilder {
    pub fn push(&mut self, command: Command) {
        self.commands.push(command);
    }

    pub fn to_player_name(&self, side: Side) -> i32 {
        if side == self.user_side {
            PlayerName::User as i32
        } else {
            PlayerName::Opponent as i32
        }
    }
}
