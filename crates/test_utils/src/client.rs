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
use data::card_state::CardState;
use data::game::{GameData, GameState, PlayerState};
use data::primitives::{CardId, GameId, RoomId, Side, UserId};
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{
    CommandList, CreateOrUpdateCardCommand, GameAction, GameRequest, PlayerName,
    UpdateGameViewCommand,
};
use server::database::Database;

#[derive(Debug, Clone)]
pub struct TestClient {
    pub data: ClientGameData,
    pub user: ClientPlayer,
    pub opponent: ClientPlayer,
    cards: HashMap<CardId, ClientCard>,
    game: GameState,
}

impl TestClient {
    /// The [UserId] for the user who the test is running as
    pub const USER_ID: UserId = UserId { value: 1 };
    /// The [UserId] for the user who is *not* running the test
    pub const OPPONENT_ID: UserId = UserId { value: 2 };
    /// The standard [GameId] used for this game
    pub const GAME_ID: GameId = GameId { value: 1 };
    /// [RoomId] used by default for targeting
    pub const ROOM_ID: RoomId = RoomId::RoomA;

    pub fn new(game: GameState, user_side: Side) -> Self {
        let (user, opponent) = match user_side {
            Side::Overlord => (&game.overlord, &game.champion),
            Side::Champion => (&game.champion, &game.overlord),
        };

        Self {
            data: ClientGameData::default(),
            user: ClientPlayer::default(),
            opponent: ClientPlayer::default(),
            cards: game.all_card_ids().map(|id| (id, ClientCard::new(game.card(id)))).collect(),
            game,
        }
    }

    /// Execute a simulated client request for this game, updating the client state as appropriate
    /// based on the responses. Returns a vector of the received commands.
    pub fn perform_action(&mut self, action: GameAction) -> Vec<Command> {
        let commands = server::handle_request(
            self,
            &GameRequest {
                action: Some(action),
                game_id: Some(protos::spelldawn::GameId { value: Self::GAME_ID.value }),
                user_id: Self::USER_ID.value,
            },
        )
        .expect("Server request failed")
        .commands
        .into_iter()
        .map(|c| c.command.expect("Empty command received"))
        .collect::<Vec<_>>();

        for command in &commands {
            self.data.update(command);
            self.user.update(command);
            self.opponent.update(command);

            for card in self.cards.values_mut() {
                card.update(command);
            }
        }

        commands
    }
}

impl Database for TestClient {
    fn generate_game_id(&self) -> Result<GameId> {
        Ok(Self::GAME_ID)
    }

    fn game(&self, id: GameId) -> Result<GameState> {
        Ok(self.game.clone())
    }

    fn write_game(&mut self, game: &GameState) -> Result<()> {
        self.game = game.clone();
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct ClientGameData {
    priority: Option<PlayerName>,
}

impl ClientGameData {
    pub fn priority(&self) -> PlayerName {
        self.priority.unwrap()
    }

    fn update(&mut self, command: &Command) {
        match command {
            Command::UpdateGameView(update_game) => {
                self.priority =
                    PlayerName::from_i32(update_game.game.as_ref().unwrap().current_priority)
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ClientPlayer {}

impl ClientPlayer {
    fn update(&mut self, command: &Command) {}
}

#[derive(Debug, Clone)]
pub struct ClientCard {}

impl ClientCard {
    pub fn new(card: &CardState) -> Self {
        Self {}
    }

    fn update(&mut self, command: &Command) {}
}
