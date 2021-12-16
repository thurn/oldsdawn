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
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{CommandList, GameCommand, RenderGameCommand};

pub mod rendering;

/// Produces a [CommandList] representing the state changes between the `old` [GameState] and the
/// `new` [GameState].
pub fn server_response(game: &GameState) -> CommandList {
    let mut commands = vec![];
    if game.modified() {
        commands.push(GameCommand {
            command: Some(Command::RenderGame(RenderGameCommand { game: None })),
        });
    }

    CommandList { commands }
}

// RenderInterfaceCommand render_interface = 4; GameState
// RenderGameCommand render_game = 5; GameState
// InitiateRaidCommand initiate_raid = 6; GameState
// EndRaidCommand end_raid = 7; GameState
// LevelUpRoomCommand level_up_room = 8; GameState
// CreateCardCommand create_card = 9; GameState
// UpdateCardCommand update_card = 10; GameState
// MoveGameObjectsCommand move_game_objects = 11; GameState
// DestroyCardCommand destroy_card = 13; GameState
// UpdatePlayerStateCommand update_player_state = 14; GameState
// DisplayGameMessageCommand display_game_message = 19; GameState
