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

use anyhow::{anyhow, bail, Context, Result};
use data::card_name::CardName;
use data::deck::Deck;
use data::game::{GameState, NewGameOptions};
use data::primitives;
use data::primitives::{Side, UserId};
use data::updates::UpdateTracker;
use display::rendering;
use maplit::hashmap;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::spelldawn_server::Spelldawn;
use protos::spelldawn::{
    card_target, CardId, CardTarget, CommandList, GameCommand, GameId, GameRequest, PlayerSide,
    RoomId,
};
use rules::actions;
use rules::actions::PlayCardTarget;
use tonic::{Request, Response, Status};
use tracing::{info, warn, warn_span};

use crate::database;

pub struct GameService {}

#[tonic::async_trait]
impl Spelldawn for GameService {
    async fn perform_action(
        &self,
        request: Request<GameRequest>,
    ) -> Result<Response<CommandList>, Status> {
        match handle_request(request.get_ref()) {
            Ok(commands) => Ok(Response::new(CommandList { commands })),
            Err(error) => {
                eprintln!("Server Error: {:#}", error);
                Err(Status::internal("Server Error"))
            }
        }
    }
}

/// Processes an incoming client request and returns a vector of [GameCommand]
/// objects describing required updates to the client UI.
fn handle_request(request: &GameRequest) -> Result<Vec<GameCommand>> {
    let game_id = to_server_game_id(&request.game_id);
    let user_id = primitives::UserId::new(request.user_id);
    let game_action = request
        .action
        .as_ref()
        .with_context(|| "Action is required")?
        .action
        .as_ref()
        .with_context(|| "GameAction is required")?;

    let span = warn_span!("handle_request", ?user_id, ?game_id, ?game_action).entered();
    warn!(?user_id, ?game_id, ?game_action, "received_request");

    let result = match game_action {
        Action::Connect(_) => handle_connect(user_id, game_id),
        Action::DrawCard(_) => handle_action(user_id, game_id, actions::draw_card),
        Action::PlayCard(action) => handle_action(user_id, game_id, |game, side| {
            actions::play_card(
                game,
                side,
                to_server_card_id(&action.card_id)?,
                card_target(&action.target),
            )
        }),
        _ => Ok(vec![]),
    }?;
    let response = result.iter().map(command_name).collect::<Vec<_>>();

    info!(?response, "sending_response");

    Ok(result)
}

fn handle_connect(
    user_id: primitives::UserId,
    game_id: Option<primitives::GameId>,
) -> Result<Vec<GameCommand>> {
    let game = if let Some(game_id) = game_id {
        database::game(game_id)?
    } else {
        let id = database::generate_id()?;
        let new_game_id = primitives::GameId::new(id);
        let mut game = GameState::new_game(
            new_game_id,
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

        game.data.turn = Side::Champion;
        game.overlord.actions = 0;
        game.champion.actions = 3;

        database::write_game(&game)?;
        info!(?new_game_id, "create_new_game");
        game
    };

    let side = user_side(user_id, &game)?;
    Ok(rendering::full_sync(&game, side))
}

fn handle_action(
    user_id: primitives::UserId,
    game_id: Option<primitives::GameId>,
    function: impl Fn(&mut GameState, Side) -> Result<()>,
) -> Result<Vec<GameCommand>> {
    let mut game = find_game(game_id)?;
    let side = user_side(user_id, &game)?;
    function(&mut game, side)?;
    let result = rendering::render_updates(&game, side);
    database::write_game(&game)?;
    Ok(result)
}

/// Look up the state for a game which is expected to exist and assigns an
/// [UpdateTracker] to it for the duration of this request.
fn find_game(game_id: Option<primitives::GameId>) -> Result<GameState> {
    let id = game_id.as_ref().with_context(|| "GameId not provided!")?;
    let mut game = database::game(*id)?;
    game.updates = UpdateTracker { update_list: Some(vec![]) };
    Ok(game)
}

/// Returns the [Side] the indicated user is representing in ths game
fn user_side(user_id: primitives::UserId, game: &GameState) -> Result<Side> {
    if user_id == game.champion.id {
        Ok(Side::Champion)
    } else if user_id == game.overlord.id {
        Ok(Side::Overlord)
    } else {
        bail!("User {:?} is not a participant in game {:?}", user_id, game.id)
    }
}

fn to_server_card_id(card_id: &Option<CardId>) -> Result<primitives::CardId> {
    if let Some(id) = card_id {
        Ok(primitives::CardId {
            side: match id.side() {
                PlayerSide::Overlord => Side::Overlord,
                PlayerSide::Champion => Side::Champion,
                _ => bail!("Invalid CardId {:?}", card_id),
            },
            index: id.index as usize,
        })
    } else {
        Err(anyhow!("Missing Required CardId"))
    }
}

fn command_name(command: &GameCommand) -> &'static str {
    if let Some(c) = &command.command {
        match c {
            Command::DebugLog(_) => "DebugLog",
            Command::RunInParallel(_) => "RunInParallel",
            Command::Delay(_) => "Delay",
            Command::RenderInterface(_) => "RenderInterface",
            Command::UpdateGameView(_) => "UpdateGameView",
            Command::InitiateRaid(_) => "InitiateRaid",
            Command::EndRaid(_) => "EndRaid",
            Command::LevelUpRoom(_) => "LevelUpRoom",
            Command::CreateOrUpdateCard(_) => "CreateOrUpdateCard",
            Command::DestroyCard(_) => "DestroyCard",
            Command::MoveGameObjects(_) => "MoveGameObjects",
            Command::MoveObjectsAtPosition(_) => "MoveObjectsAtPosition",
            Command::PlaySound(_) => "PlaySound",
            Command::SetMusic(_) => "SetMusic",
            Command::FireProjectile(_) => "FireProjectile",
            Command::PlayEffect(_) => "PlayEffect",
            Command::DisplayGameMessage(_) => "DisplayGameMessage",
            Command::SetGameObjectsEnabled(_) => "SetGameObjectsEnabled",
            Command::DisplayRewards(_) => "DisplayRewards",
        }
    } else {
        "None"
    }
}

fn card_target(target: &Option<CardTarget>) -> PlayCardTarget {
    target.as_ref().map_or(PlayCardTarget::None, |t| {
        t.card_target.as_ref().map_or(PlayCardTarget::None, |t2| match t2 {
            card_target::CardTarget::RoomId(room_id) => {
                to_server_room_id(RoomId::from_i32(*room_id))
                    .map_or(PlayCardTarget::None, PlayCardTarget::Room)
            }
        })
    })
}

fn to_server_game_id(game_id: &Option<GameId>) -> Option<primitives::GameId> {
    game_id.as_ref().map(|g| primitives::GameId::new(g.value))
}

fn to_server_room_id(room_id: Option<RoomId>) -> Option<primitives::RoomId> {
    match room_id {
        None | Some(RoomId::Unspecified) => None,
        Some(RoomId::Vault) => Some(primitives::RoomId::Vault),
        Some(RoomId::Sanctum) => Some(primitives::RoomId::Sanctum),
        Some(RoomId::Crypts) => Some(primitives::RoomId::Crypts),
        Some(RoomId::RoomA) => Some(primitives::RoomId::RoomA),
        Some(RoomId::RoomB) => Some(primitives::RoomId::RoomB),
        Some(RoomId::RoomC) => Some(primitives::RoomId::RoomC),
        Some(RoomId::RoomD) => Some(primitives::RoomId::RoomD),
        Some(RoomId::RoomE) => Some(primitives::RoomId::RoomE),
    }
}
