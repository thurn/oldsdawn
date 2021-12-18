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
use maplit::hashmap;
use model::card_name::CardName;
use model::deck::Deck;
use model::game::{GameState, NewGameOptions};
use model::primitives;
use once_cell::sync::Lazy;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::spelldawn_server::{Spelldawn, SpelldawnServer};
use protos::spelldawn::{
    CommandList, ConnectAction, GameCommand, GameId, GameRequest, GameView, UpdateGameViewCommand,
};
use sled::{Db, IVec, Tree};
use tonic::{transport::Server, Code, Request, Response, Status};

use crate::database;

#[derive(Default)]
pub struct GameService {}

#[tonic::async_trait]
impl Spelldawn for GameService {
    async fn perform_action(
        &self,
        request: Request<GameRequest>,
    ) -> Result<Response<CommandList>, Status> {
        println!("Got a request from {:?}", request.remote_addr());
        Ok(Response::new(CommandList {
            commands: handle_request(request.get_ref())
                .map_err(|err| Status::internal("Server Error"))?,
        }))
    }
}

/// Processes an incoming client request and returns a vector of [GameCommand] objects describing
/// required updates to the client UI.
fn handle_request(request: &GameRequest) -> Result<Vec<GameCommand>> {
    match request
        .action
        .as_ref()
        .with_context(|| "Action is required")?
        .action
        .as_ref()
        .with_context(|| "GameAction is required")?
    {
        Action::Connect(action) => handle_connect(action),
        _ => Ok(vec![]),
    }
}

fn handle_connect(action: &ConnectAction) -> Result<Vec<GameCommand>> {
    if let Some(game_id) = &action.game_id {
        let game = database::game(primitives::GameId::new(game_id.value))?;
    } else {
        let id = database::generate_id()?;
        let game = GameState::new_game(
            primitives::GameId::new(id),
            Deck {
                identity: CardName::TestOverlordIdentity,
                cards: hashmap! {
                    CardName::DungeonAnnex => 45,
                },
            },
            Deck {
                identity: CardName::TestChampionIdentity,
                cards: hashmap! {
                    CardName::ArcaneRecovery => 45,
                },
            },
            NewGameOptions::default(),
        );
        database::write_game(&game)?;
    }

    Ok(vec![])
}
