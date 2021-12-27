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
use dashmap::DashMap;
use data::card_name::CardName;
use data::deck::Deck;
use data::game::{GameState, NewGameOptions};
use data::primitives::{self, CardId, GameId, RoomId};
use data::primitives::{Side, UserId};
use data::updates::UpdateTracker;
use display::rendering;
use maplit::hashmap;
use once_cell::sync::Lazy;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::spelldawn_server::Spelldawn;
use protos::spelldawn::{
    card_target, CardIdentifier, CardTarget, CommandList, ConnectRequest, GameCommand,
    GameIdentifier, GameRequest, PlayerSide, RoomIdentifier,
};
use rules::actions;
use rules::actions::PlayCardTarget;
use tokio::sync::mpsc::Sender;
use tokio::sync::{mpsc};
use tokio_stream::wrappers::ReceiverStream;

use tonic::{Request, Response, Status};
use tracing::{info, warn, warn_span};

use crate::database::{Database, SledDatabase};

pub mod database;

static CHANNELS: Lazy<DashMap<UserId, Sender<Result<CommandList, Status>>>> =
    Lazy::new(DashMap::new);

pub struct GameService {}

#[tonic::async_trait]
impl Spelldawn for GameService {
    type ConnectStream = ReceiverStream<Result<CommandList, Status>>;

    async fn connect(
        &self,
        request: Request<ConnectRequest>,
    ) -> Result<Response<Self::ConnectStream>, Status> {
        let message = request.get_ref();
        let game_id = to_server_game_id(&message.game_id);
        let user_id = primitives::UserId::new(message.user_id);
        warn!(?user_id, ?game_id, "received_connection");

        let (tx, rx) = mpsc::channel(4);
        CHANNELS.insert(user_id, tx);
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn perform_action(
        &self,
        request: Request<GameRequest>,
    ) -> Result<Response<CommandList>, Status> {
        let mut database = SledDatabase;
        match handle_request(&mut database, request.get_ref()) {
            Ok(response) => {
                for (user_id, commands) in response.channel_responses {
                    if let Some(channel) = CHANNELS.get(&user_id) {
                        if let Err(e) = channel.send(Ok(commands)).await {
                            return Err(Status::internal(format!("Channel Error: {:#}", e)));
                        }
                    }
                }
                Ok(Response::new(response.command_list))
            }
            Err(error) => {
                eprintln!("Server Error: {:#}", error);
                Err(Status::internal(format!("Server Error: {:#}", error)))
            }
        }
    }
}

/// A response to a given [GameRequest] which should be sent to a specific user. Returned from
/// [handle_request] to support providing updates to different players in a game.
#[derive(Debug, Clone, Default)]
pub struct GameResponse {
    /// Response to send to the user who made the initial game request.
    pub command_list: CommandList,
    /// Responses to send to other users, e.g. to update opponent state.
    pub channel_responses: Vec<(UserId, CommandList)>,
}

/// Processes an incoming client request and returns a vector of [UserResponse] objects describing
/// required updates to send to connected users.
pub fn handle_request(database: &mut impl Database, request: &GameRequest) -> Result<GameResponse> {
    let game_id = to_server_game_id(&request.game_id);
    let user_id = UserId::new(request.user_id);
    let game_action = request
        .action
        .as_ref()
        .with_context(|| "Action is required")?
        .action
        .as_ref()
        .with_context(|| "GameAction is required")?;

    let _span = warn_span!("handle_request", ?user_id, ?game_id, ?game_action).entered();
    warn!(?user_id, ?game_id, ?game_action, "received_request");

    let response = match game_action {
        Action::Connect(_) => handle_connect(database, user_id, game_id),
        Action::DrawCard(_) => handle_action(database, user_id, game_id, actions::draw_card),
        Action::PlayCard(action) => handle_action(database, user_id, game_id, |game, side| {
            actions::play_card(
                game,
                side,
                to_server_card_id(&action.card_id)?,
                card_target(&action.target),
            )
        }),
        _ => Ok(GameResponse::default()),
    }?;

    let commands = response.command_list.commands.iter().map(command_name).collect::<Vec<_>>();
    info!(?user_id, ?commands, "sending_response");

    Ok(response)
}

fn handle_connect(
    database: &mut impl Database,
    user_id: primitives::UserId,
    game_id: Option<primitives::GameId>,
) -> Result<GameResponse> {
    let game = if let Some(game_id) = game_id {
        database.game(game_id)?
    } else {
        let new_game_id = database.generate_game_id()?;
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

        database.write_game(&game)?;
        info!(?new_game_id, "create_new_game");
        game
    };

    let side = user_side(user_id, &game)?;
    Ok(GameResponse {
        command_list: CommandList { commands: rendering::full_sync(&game, side) },
        channel_responses: vec![],
    })
}

fn handle_action(
    database: &mut impl Database,
    user_id: UserId,
    game_id: Option<GameId>,
    function: impl Fn(&mut GameState, Side) -> Result<()>,
) -> Result<GameResponse> {
    let mut game = find_game(database, game_id)?;
    let side = user_side(user_id, &game)?;
    function(&mut game, side)?;

    let user_result = rendering::render_updates(&game, side);
    let opponent_id = game.player(side.opponent()).id;
    let opponent_result = rendering::render_updates(&game, side.opponent());

    database.write_game(&game)?;
    Ok(GameResponse {
        command_list: CommandList { commands: user_result },
        channel_responses: vec![(opponent_id, CommandList { commands: opponent_result })],
    })
}

/// Look up the state for a game which is expected to exist and assigns an
/// [UpdateTracker] to it for the duration of this request.
fn find_game(database: &impl Database, game_id: Option<GameId>) -> Result<GameState> {
    let id = game_id.as_ref().with_context(|| "GameId not provided!")?;
    let mut game = database.game(*id)?;
    game.updates = UpdateTracker { update_list: Some(vec![]) };
    Ok(game)
}

/// Returns the [Side] the indicated user is representing in this game
pub fn user_side(user_id: UserId, game: &GameState) -> Result<Side> {
    if user_id == game.champion.id {
        Ok(Side::Champion)
    } else if user_id == game.overlord.id {
        Ok(Side::Overlord)
    } else {
        bail!("User {:?} is not a participant in game {:?}", user_id, game.id)
    }
}

fn to_server_card_id(card_id: &Option<CardIdentifier>) -> Result<CardId> {
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
    command.command.as_ref().map_or("None", |c| match c {
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
    })
}

fn card_target(target: &Option<CardTarget>) -> PlayCardTarget {
    target.as_ref().map_or(PlayCardTarget::None, |t| {
        t.card_target.as_ref().map_or(PlayCardTarget::None, |t2| match t2 {
            card_target::CardTarget::RoomId(room_id) => {
                to_server_room_id(RoomIdentifier::from_i32(*room_id))
                    .map_or(PlayCardTarget::None, PlayCardTarget::Room)
            }
        })
    })
}

fn to_server_game_id(game_id: &Option<GameIdentifier>) -> Option<GameId> {
    game_id.as_ref().map(|g| primitives::GameId::new(g.value))
}

fn to_server_room_id(room_id: Option<RoomIdentifier>) -> Option<RoomId> {
    match room_id {
        None | Some(RoomIdentifier::Unspecified) => None,
        Some(RoomIdentifier::Vault) => Some(RoomId::Vault),
        Some(RoomIdentifier::Sanctum) => Some(RoomId::Sanctum),
        Some(RoomIdentifier::Crypts) => Some(RoomId::Crypts),
        Some(RoomIdentifier::RoomA) => Some(RoomId::RoomA),
        Some(RoomIdentifier::RoomB) => Some(RoomId::RoomB),
        Some(RoomIdentifier::RoomC) => Some(RoomId::RoomC),
        Some(RoomIdentifier::RoomD) => Some(RoomId::RoomD),
        Some(RoomIdentifier::RoomE) => Some(RoomId::RoomE),
    }
}
