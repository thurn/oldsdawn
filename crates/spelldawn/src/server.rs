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
use data::card_name::CardName;
use data::deck::Deck;
use data::game::{GameState, NewGameOptions};
use data::primitives;
use data::primitives::{Side, UserId};
use display::rendering;
use maplit::hashmap;
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
use crate::database::game;

#[derive(Default)]
pub struct GameService {}

#[tonic::async_trait]
impl Spelldawn for GameService {
    async fn perform_action(
        &self,
        request: Request<GameRequest>,
    ) -> Result<Response<CommandList>, Status> {
        Ok(Response::new(CommandList {
            commands: handle_request(request.get_ref())
                .map_err(|err| Status::internal("Server Error"))?,
        }))
    }
}

/// Processes an incoming client request and returns a vector of [GameCommand] objects describing
/// required updates to the client UI.
fn handle_request(request: &GameRequest) -> Result<Vec<GameCommand>> {
    let game_id = &request.game_id;
    let user_id = &request.user_id;
    let game_action = request
        .action
        .as_ref()
        .with_context(|| "Action is required")?
        .action
        .as_ref()
        .with_context(|| "GameAction is required")?;
    println!("Got request in game {:?} from user {:?}: {:?}", game_id, user_id, game_action);
    match game_action {
        Action::Connect(action) => handle_connect(*user_id, game_id),
        _ => Ok(vec![]),
    }
}

fn handle_connect(user_id: u64, game_id: &Option<GameId>) -> Result<Vec<GameCommand>> {
    let game = if let Some(game_id) = game_id {
        database::game(primitives::GameId::new(game_id.value))?
    } else {
        let id = database::generate_id()?;
        let game = GameState::new_game(
            primitives::GameId::new(id),
            Deck {
                owner_id: UserId::new(2),
                identity: CardName::TestOverlordIdentity,
                cards: hashmap! {
                    CardName::DungeonAnnex => 45,
                },
            },
            Deck {
                owner_id: UserId::new(1),
                identity: CardName::TestChampionIdentity,
                cards: hashmap! {
                    CardName::ArcaneRecovery => 45,
                },
            },
            NewGameOptions::default(),
        );
        database::write_game(&game)?;
        game
    };

    let side = user_side(user_id, &game);
    Ok(rendering::full_sync(&game, side))
}

/// Returns the [Side] the indicated user is representing in ths game
fn user_side(user_id: u64, game: &GameState) -> Side {
    if user_id == game.champion.id.value {
        Side::Champion
    } else if user_id == game.overlord.id.value {
        Side::Overlord
    } else {
        panic!("Player {:?} is not a participant in game {:?}", user_id, game.id)
    }
}
