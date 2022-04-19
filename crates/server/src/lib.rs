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

//! Top-level server response handling

use anyhow::{bail, ensure, Context, Result};
use dashmap::DashMap;
use data::game::{GameConfiguration, GameState};
use data::game_actions;
use data::game_actions::UserAction;
use data::primitives::{GameId, PlayerId, RoomId, Side};
use data::updates::UpdateTracker;
use data::with_error::WithError;
use display::adapters::ServerCardId;
use display::{adapters, render};
use enum_iterator::IntoEnumIterator;
use once_cell::sync::Lazy;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::spelldawn_server::Spelldawn;
use protos::spelldawn::{
    card_target, CardTarget, CommandList, ConnectRequest, ConnectToGameCommand,
    CreateNewGameAction, GameCommand, GameIdentifier, GameRequest, PlayerSide, RoomIdentifier,
    StandardAction,
};
use rules::{actions, mutations, queries};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio::task;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{error, info, warn, warn_span};

use crate::database::{Database, SledDatabase};

pub mod database;
pub mod debug;

/// Stores active channels for each user.
///
/// TODO: How do you clean these up if a user disconnects?
static CHANNELS: Lazy<DashMap<PlayerId, Sender<Result<CommandList, Status>>>> =
    Lazy::new(DashMap::new);

/// Struct which implements our GRPC service
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
        let player_id = match adapters::to_server_player_id(message.player_id) {
            Ok(player_id) => player_id,
            _ => return Err(Status::unauthenticated("PlayerId is required")),
        };
        warn!(?player_id, ?game_id, "received_connection");

        let (tx, rx) = mpsc::channel(4);

        let mut database = SledDatabase;
        match handle_connect(&mut database, player_id, game_id) {
            Ok(commands) => {
                let names = commands.commands.iter().map(command_name).collect::<Vec<_>>();
                info!(?player_id, ?names, "sending_connection_response");

                if let Err(error) = tx.send(Ok(commands)).await {
                    error!(?player_id, ?game_id, ?error, "Send Error!");
                    return Err(Status::internal(format!("Send Error: {:#}", error)));
                }
            }
            Err(error) => {
                error!(?player_id, ?game_id, ?error, "Connection Error!");
                return Err(Status::internal(format!("Connection Error: {:#}", error)));
            }
        }

        CHANNELS.insert(player_id, tx);
        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn perform_action(
        &self,
        request: Request<GameRequest>,
    ) -> Result<Response<CommandList>, Status> {
        let mut db = SledDatabase {};
        match handle_request(&mut db, request.get_ref()) {
            Ok(response) => {
                send_player_response(response.opponent_response).await;
                check_for_agent_response(db, request.get_ref()).expect("AI Agent Error");
                Ok(Response::new(response.command_list))
            }
            Err(error) => {
                error!(?error, "Server Error!");
                Err(Status::internal(format!("Server Error: {:#}", error)))
            }
        }
    }
}

/// A response to a given [GameRequest].
///
/// Returned from [handle_request] to support providing updates to different
/// players in a game.
#[derive(Debug, Clone, Default)]
pub struct GameResponse {
    /// Response to send to the user who made the initial game request.
    pub command_list: CommandList,
    /// Response to send to update opponent state.
    pub opponent_response: Option<(PlayerId, CommandList)>,
}

impl GameResponse {
    pub fn from_commands(command_list: Vec<Command>) -> Self {
        Self {
            command_list: CommandList {
                commands: command_list
                    .into_iter()
                    .map(|c| GameCommand { command: Some(c) })
                    .collect(),
            },
            opponent_response: None,
        }
    }
}

/// Processes an incoming client request and returns a [GameResponse] describing
/// required updates to send to connected users.
pub fn handle_request(database: &mut impl Database, request: &GameRequest) -> Result<GameResponse> {
    let game_id = to_server_game_id(&request.game_id);
    let player_id = adapters::to_server_player_id(request.player_id)?;
    let game_action = request
        .action
        .as_ref()
        .with_context(|| "Action is required")?
        .action
        .as_ref()
        .with_context(|| "GameAction is required")?;

    let _span = warn_span!("handle_request", ?player_id, ?game_id, ?game_action).entered();
    warn!(?player_id, ?game_id, ?game_action, "received_request");

    let response = match game_action {
        Action::StandardAction(standard_action) => {
            handle_standard_action(database, player_id, game_id, standard_action)
        }
        Action::TogglePanel(toggle_panel) => {
            Ok(GameResponse::from_commands(vec![Command::RenderInterface(panels::render_panel(
                toggle_panel.panel_address(),
            )?)]))
        }
        Action::CreateNewGame(create_game) => create_new_game(database, player_id, create_game),
        Action::DrawCard(_) => handle_action(database, player_id, game_id, UserAction::DrawCard),
        Action::PlayCard(action) => {
            let action = match adapters::to_server_card_id(action.card_id)? {
                ServerCardId::CardId(card_id) => {
                    UserAction::PlayCard(card_id, card_target(&action.target))
                }
                ServerCardId::AbilityId(ability_id) => {
                    UserAction::ActivateAbility(ability_id, card_target(&action.target))
                }
            };
            handle_action(database, player_id, game_id, action)
        }
        Action::GainMana(_) => handle_action(database, player_id, game_id, UserAction::GainMana),
        Action::InitiateRaid(action) => {
            let room_id = adapters::to_server_room_id(action.room_id)?;
            handle_action(database, player_id, game_id, UserAction::InitiateRaid(room_id))
        }
        Action::LevelUpRoom(level_up) => {
            let room_id = adapters::to_server_room_id(level_up.room_id)?;
            handle_action(database, player_id, game_id, UserAction::LevelUpRoom(room_id))
        }
        Action::SpendActionPoint(_) => {
            handle_action(database, player_id, game_id, UserAction::SpendActionPoint)
        }
    }?;

    let commands = response.command_list.commands.iter().map(command_name).collect::<Vec<_>>();
    info!(?player_id, ?commands, "sending_response");

    Ok(response)
}

/// Sets up the game state for a game connection request, connecting to
/// the `game_id` game.
pub fn handle_connect(
    database: &mut impl Database,
    player_id: PlayerId,
    game_id: Option<GameId>,
) -> Result<CommandList> {
    if let Some(game_id) = game_id {
        if database.has_game(game_id)? {
            let game = database.game(game_id)?;
            let side = user_side(player_id, &game)?;
            let mut commands = render::connect(&game, side);
            panels::render_standard_panels(&mut commands)?;
            Ok(command_list(commands))
        } else {
            bail!("Game not found: {:?}", game_id)
        }
    } else {
        Ok(command_list(vec![]))
    }
}

/// Creates a new default [GameState], deals opening hands, and writes its value
/// to the database.
fn create_new_game(
    database: &mut impl Database,
    user_id: PlayerId,
    action: &CreateNewGameAction,
) -> Result<GameResponse> {
    let game_id = if action.use_debug_id { GameId::new(0) } else { database.generate_game_id()? };
    info!(?game_id, "create_new_game");
    let opponent_id = adapters::to_server_player_id(action.opponent_id)?;
    let user_side = adapters::to_server_side(PlayerSide::from_i32(action.side))?;
    let (overlord_deck, champion_deck) = match user_side {
        Side::Overlord => {
            (database.deck(user_id, Side::Overlord)?, database.deck(opponent_id, Side::Champion)?)
        }
        Side::Champion => {
            (database.deck(opponent_id, Side::Overlord)?, database.deck(user_id, Side::Champion)?)
        }
    };

    let mut game = GameState::new(
        game_id,
        overlord_deck,
        champion_deck,
        GameConfiguration { deterministic: action.deterministic, ..GameConfiguration::default() },
    );

    mutations::deal_opening_hands(&mut game);
    database.write_game(&game)?;
    let commands = command_list(vec![Command::ConnectToGame(ConnectToGameCommand {
        game_id: Some(adapters::adapt_game_id(game.id)),
        scene_name: "Labyrinth".to_string(),
    })]);

    Ok(GameResponse {
        command_list: commands.clone(),
        opponent_response: Some((opponent_id, commands)),
    })
}

/// Queries the [GameState] for a game from the [Database] and then invokes the
/// [actions::handle_user_action] function to apply the provided [UserAction].
///
/// Converts the resulting [GameState] into a series of client updates for both
/// players in the form of a [GameResponse] and then writes the new game state
/// back to the database
///
/// Schedules an AI Agent response if one is required for the current game
/// state.
fn handle_action(
    database: &mut impl Database,
    player_id: PlayerId,
    game_id: Option<GameId>,
    action: UserAction,
) -> Result<GameResponse> {
    handle_custom_action(database, player_id, game_id, |game, user_side| {
        actions::handle_user_action(game, user_side, action)
    })
}

/// Custom version of `handle_action` which accepts a function allowing
/// arbitrary mutation of the [GameState].
pub fn handle_custom_action(
    database: &mut impl Database,
    player_id: PlayerId,
    game_id: Option<GameId>,
    function: impl Fn(&mut GameState, Side) -> Result<()>,
) -> Result<GameResponse> {
    let mut game = find_game(database, game_id)?;
    let user_side = user_side(player_id, &game)?;
    function(&mut game, user_side)?;

    let user_result = render::render_updates(&game, user_side);
    let opponent_id = game.player(user_side.opponent()).id;

    let channel_response =
        Some((opponent_id, command_list(render::render_updates(&game, user_side.opponent()))));
    database.write_game(&game)?;

    Ok(GameResponse {
        command_list: command_list(user_result),
        opponent_response: channel_response,
    })
}

pub fn check_for_agent_response(mut database: SledDatabase, request: &GameRequest) -> Result<()> {
    if let Some(game_id) = to_server_game_id(&request.game_id) {
        if database.has_game(game_id)? {
            task::spawn(async move {
                loop {
                    let mut took_action = false;
                    let game = database.game(game_id).expect("game");
                    for side in Side::into_enum_iter() {
                        if let Some(agent_data) = game.player(side).agent {
                            if queries::can_take_action(&game, side) {
                                took_action = true;
                                let agent = ai::core::get_agent(agent_data.name);
                                let state_predictor =
                                    ai::core::get_game_state_predictor(agent_data.state_predictor);
                                let action = agent(state_predictor(&game, side), side)
                                    .expect("Error invoking AI Agent");
                                let response = handle_action(
                                    &mut database,
                                    game.player(side).id,
                                    Some(game_id),
                                    action,
                                )
                                .expect("Error handling GameAction");
                                send_player_response(response.opponent_response).await;
                                send_player_response(Some((
                                    game.player(side).id,
                                    response.command_list,
                                )))
                                .await
                            }
                        }
                    }

                    if !took_action {
                        break;
                    }
                }
            });
        }
    }

    Ok(())
}

/// Sends a game response to a given player, if they are connected to the
/// server.
async fn send_player_response(response: Option<(PlayerId, CommandList)>) {
    if let Some((player_id, commands)) = response {
        if let Some(channel) = CHANNELS.get(&player_id) {
            if channel.send(Ok(commands)).await.is_err() {
                // This returns SendError if the client is disconnected, which isn't a
                // huge problem. Hopefully they will reconnect again in the future.
                info!(?player_id, "client_is_disconnected");
            }
        }
    }
}

/// Parses the serialized payload in a [StandardAction] and dispatches to the
/// correct handler.
fn handle_standard_action(
    database: &mut impl Database,
    player_id: PlayerId,
    game_id: Option<GameId>,
    standard_action: &StandardAction,
) -> Result<GameResponse> {
    ensure!(!standard_action.payload.is_empty(), "Empty action payload received");
    let action: UserAction = bincode::deserialize(&standard_action.payload)
        .with_error(|| "Failed to deserialize action payload")?;
    match action {
        UserAction::Debug(debug_action) => {
            debug::handle_debug_action(database, player_id, game_id, debug_action)
        }
        _ => handle_action(database, player_id, game_id, action),
    }
}

// fn handle_agent_response(game: GameState, side: Side, agent_data: AgentData)
// {     task::spawn(async move {
//         sleep(Duration::from_millis(1000)).await;
//         let agent = agents::core::get_agent(agent_data.name);
//         let state_predictor =
// agents::core::get_game_state_predictor(agent_data.state_predictor);
//         let states = state_predictor(&game, side);
//         let _action = agent(states, side).expect("Error invoking AI Agent");
//         // let response = handle_action(database, game.player(side).id,
//         // Some(game.id), action);
//     });
// }

/// Look up the state for a game which is expected to exist and assigns an
/// [UpdateTracker] to it for the duration of this request.
fn find_game(database: &impl Database, game_id: Option<GameId>) -> Result<GameState> {
    let id = game_id.as_ref().with_context(|| "GameId not provided!")?;
    let mut game = database.game(*id)?;
    game.updates = UpdateTracker::new(!game.data.config.simulation);
    Ok(game)
}

/// Returns the [Side] the indicated user is representing in this game
pub fn user_side(player_id: PlayerId, game: &GameState) -> Result<Side> {
    if player_id == game.champion.id {
        Ok(Side::Champion)
    } else if player_id == game.overlord.id {
        Ok(Side::Overlord)
    } else {
        bail!("User {:?} is not a participant in game {:?}", player_id, game.id)
    }
}

/// Get a display name for a command. Used for debugging.
pub fn command_name(command: &GameCommand) -> &'static str {
    command.command.as_ref().map_or("None", |c| match c {
        Command::Debug(_) => "Debug",
        Command::RunInParallel(_) => "RunInParallel",
        Command::Delay(_) => "Delay",
        Command::ConnectToGame(_) => "ConnectToGame",
        Command::RenderInterface(_) => "RenderInterface",
        Command::TogglePanel(_) => "TogglePanel",
        Command::UpdateGameView(_) => "UpdateGameView",
        Command::VisitRoom(_) => "VisitRoom",
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
        Command::LoadScene(_) => "LoadScene",
        Command::SetPlayerId(_) => "SetPlayerIdentifier",
    })
}

fn card_target(target: &Option<CardTarget>) -> game_actions::CardTarget {
    target.as_ref().map_or(game_actions::CardTarget::None, |t| {
        t.card_target.as_ref().map_or(game_actions::CardTarget::None, |t2| match t2 {
            card_target::CardTarget::RoomId(room_id) => {
                to_server_room_id(RoomIdentifier::from_i32(*room_id))
                    .map_or(game_actions::CardTarget::None, game_actions::CardTarget::Room)
            }
        })
    })
}

fn command_list(commands: Vec<Command>) -> CommandList {
    CommandList {
        commands: commands.into_iter().map(|c| GameCommand { command: Some(c) }).collect(),
    }
}

fn to_server_game_id(game_id: &Option<GameIdentifier>) -> Option<GameId> {
    game_id.as_ref().map(|g| GameId::new(g.value))
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
