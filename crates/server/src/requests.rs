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

//! Top-level server request handling

use actions;
use adapters::ServerCardId;
use anyhow::Result;
use cards::decklists;
use dashmap::DashMap;
use data::deck::Deck;
use data::game::{GameConfiguration, GameState};
use data::game_actions::UserAction;
use data::player_data::{CurrentGame, NewGameRequest};
use data::player_name::PlayerId;
use data::primitives::{GameId, Side};
use data::updates::{UpdateTracker, Updates};
use data::{game_actions, player_data};
use display::render;
use once_cell::sync::Lazy;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::spelldawn_server::Spelldawn;
use protos::spelldawn::{
    card_target, CardTarget, CommandList, ConnectRequest, GameCommand, GameRequest,
    LoadSceneCommand, NewGameAction, PlayerIdentifier, SceneLoadMode, StandardAction,
};
use rules::{dispatch, mutations};
use serde_json::de;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{error, info, warn, warn_span};
use with_error::{fail, verify, WithError};

use crate::agent_response::HandleRequest;
use crate::database::{Database, SledDatabase};
use crate::{agent_response, debug};

/// Stores active channels for each user.
///
/// TODO: Clean this up on disconnect. This is quite easy to do with 'real' gRPC
/// but I haven't figured out how to do it with gRPC-web (which is just
/// fake-streaming over HTTP1). Unity doesn't support HTTP2 natively, but it's
/// possible to do it via a third party networking stack.
static CHANNELS: Lazy<DashMap<PlayerId, Sender<Result<CommandList, Status>>>> =
    Lazy::new(DashMap::new);

pub type ResponseInterceptor = fn(&CommandList);

/// Struct which implements our GRPC service
pub struct GameService {
    pub response_interceptor: Option<ResponseInterceptor>,
}

#[tonic::async_trait]
impl Spelldawn for GameService {
    type ConnectStream = ReceiverStream<Result<CommandList, Status>>;

    async fn connect(
        &self,
        request: Request<ConnectRequest>,
    ) -> Result<Response<Self::ConnectStream>, Status> {
        let mut db = SledDatabase { flush_on_write: false };
        let message = request.get_ref();
        let player_id = match player_id(&mut db, &message.player_id) {
            Ok(player_id) => player_id,
            _ => return Err(Status::unauthenticated("PlayerId is required")),
        };
        warn!(?player_id, "received_connection");

        let (tx, rx) = mpsc::channel(4);

        let result = handle_connect(&mut db, player_id);
        match result {
            Ok(commands) => {
                let names = commands.commands.iter().map(command_name).collect::<Vec<_>>();
                info!(?player_id, ?names, "sending_connection_response");

                if let Err(error) = tx.send(Ok(commands)).await {
                    error!(?player_id, ?error, "Send Error!");
                    return Err(Status::internal(format!("Send Error: {:#}", error)));
                }
            }
            Err(error) => {
                error!(?player_id, ?error, "Connection Error!");
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
        let mut db = SledDatabase { flush_on_write: false };
        let response = handle_request(&mut db, request.get_ref());
        match response {
            Ok(response) => {
                if let Some(interceptor) = self.response_interceptor {
                    interceptor(&response.command_list);
                }

                send_player_response(response.opponent_response).await;
                let result = agent_response::handle_request(
                    db,
                    request.get_ref(),
                    HandleRequest::SendToPlayer,
                );
                if let Err(error) = result {
                    return Err(Status::internal(format!("Agent Error: {:#}", error)));
                }
                Ok(Response::new(response.command_list))
            }
            Err(error) => {
                error!(?error, "Server Error!");
                Err(Status::internal(format!("Server Error: {:#}", error)))
            }
        }
    }
}

/// Helper to perform the connect action from the unity plugin
pub fn connect(message: ConnectRequest) -> Result<CommandList> {
    let mut db = SledDatabase { flush_on_write: true };
    let player_id = player_id(&mut db, &message.player_id)?;
    handle_connect(&mut db, player_id)
}

/// Helper to perform an action from the unity plugin
pub fn perform_action(request: GameRequest) -> Result<CommandList> {
    let mut db = SledDatabase { flush_on_write: true };
    let response = handle_request(&mut db, &request)?;
    agent_response::handle_request(db, &request, HandleRequest::PushQueue)?;
    Ok(response.command_list)
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
    let player_id = player_id(database, &request.player_id)?;
    let game_id = player_data::current_game_id(database.player(player_id)?);
    let game_action = request
        .action
        .as_ref()
        .with_error(|| "Action is required")?
        .action
        .as_ref()
        .with_error(|| "GameAction is required")?;

    let _span = warn_span!("handle_request", ?player_id, ?game_id, ?game_action).entered();
    warn!(?player_id, ?game_id, ?game_action, "received_request");

    let response = match game_action {
        Action::StandardAction(standard_action) => {
            handle_standard_action(database, player_id, game_id, standard_action)
        }
        Action::FetchPanel(fetch_panel) => {
            Ok(GameResponse::from_commands(vec![Command::UpdatePanels(panels::render_panel(
                fetch_panel.panel_address.clone().with_error(|| "missing address")?,
            )?)]))
        }
        Action::NewGame(create_game) => handle_new_game(database, player_id, create_game),
        Action::DrawCard(_) => handle_action(database, player_id, game_id, UserAction::DrawCard),
        Action::PlayCard(action) => {
            let action =
                match adapters::server_card_id(action.card_id.with_error(|| "CardID expected")?)? {
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
            let room_id = adapters::room_id(action.room_id)?;
            handle_action(database, player_id, game_id, UserAction::InitiateRaid(room_id))
        }
        Action::LevelUpRoom(level_up) => {
            let room_id = adapters::room_id(level_up.room_id)?;
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

/// Sets up the game state for a game connection request.
pub fn handle_connect(database: &mut impl Database, player_id: PlayerId) -> Result<CommandList> {
    if let Some(game_id) = player_data::current_game_id(database.player(player_id)?) {
        if database.has_game(game_id)? {
            let game = database.game(game_id)?;
            let side = user_side(player_id, &game)?;
            let mut commands = render::connect(&game, side)?;
            panels::append_standard_panels(&mut commands)?;
            Ok(command_list(commands))
        } else {
            fail!("Game not found: {:?}", game_id)
        }
    } else {
        Ok(command_list(vec![]))
    }
}

/// Creates a new default [GameState], deals opening hands, and writes its value
/// to the database.
fn handle_new_game(
    database: &mut impl Database,
    user_id: PlayerId,
    action: &NewGameAction,
) -> Result<GameResponse> {
    let debug_options = action.debug_options.clone().unwrap_or_default();
    let opponent_id = player_id(database, &action.opponent_id)?;
    let deck_id = adapters::deck_id(action.deck.with_error(|| "Expected Deck ID")?);
    let mut user = database.player(user_id)?.with_error(|| "User not found")?;
    let user_deck = user.deck(deck_id).clone();
    let opponent_deck =
        if let Some(deck) = requested_deck(database, opponent_id, user_deck.side.opponent())? {
            deck
        } else {
            user.current_game = Some(CurrentGame::Requested(NewGameRequest { deck_id }));
            database.write_player(&user)?;
            // TODO: Implement some kind of waiting UI here
            return Ok(GameResponse::from_commands(vec![]));
        };

    let (overlord_deck, champion_deck) = match (user_deck.side, opponent_deck.side) {
        (Side::Overlord, Side::Champion) => (user_deck, opponent_deck),
        (Side::Champion, Side::Overlord) => (opponent_deck, user_deck),
        _ => fail!("Deck side mismatch!"),
    };
    let game_id = debug_options
        .override_game_identifier
        .map_or(database.generate_game_id()?, adapters::game_id);
    info!(?game_id, "create_new_game");

    let mut game = GameState::new(
        game_id,
        overlord_deck,
        champion_deck,
        GameConfiguration {
            deterministic: debug_options.deterministic,
            ..GameConfiguration::default()
        },
    );

    dispatch::populate_delegate_cache(&mut game);
    mutations::deal_opening_hands(&mut game)?;
    database.write_game(&game)?;

    user.current_game = Some(CurrentGame::Playing(game_id));
    database.write_player(&user)?;

    if let PlayerId::Database(_) = opponent_id {
        let mut opponent = database.player(opponent_id)?.with_error(|| "Opponent not found")?;
        opponent.current_game = Some(CurrentGame::Playing(game_id));
        database.write_player(&opponent)?;
    }

    let commands = command_list(vec![Command::LoadScene(LoadSceneCommand {
        scene_name: "Labyrinth".to_string(),
        mode: SceneLoadMode::Single as i32,
    })]);

    Ok(GameResponse {
        command_list: commands.clone(),
        opponent_response: Some((opponent_id, commands)),
    })
}

/// Looks up the deck the `player_id` player has requested to use for a new game
fn requested_deck(
    database: &impl Database,
    player_id: PlayerId,
    side: Side,
) -> Result<Option<Deck>> {
    Ok(match player_id {
        PlayerId::Database(_) => {
            let player = database.player(player_id)?.with_error(|| "Player not found")?;
            player.requested_deck_id().map(|deck_id| player.deck(deck_id).clone())
        }
        // TODO: Each named player should have their own decklist
        PlayerId::Named(_) => Some(decklists::canonical_deck(player_id, side)),
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
pub fn handle_action(
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
    // TODO: Use transactions?
    let mut game = find_game(database, game_id)?;
    let user_side = user_side(player_id, &game)?;
    function(&mut game, user_side)?;

    let user_result = render::render_updates(&game, user_side)?;
    let opponent_id = game.player(user_side.opponent()).id;

    let channel_response =
        Some((opponent_id, command_list(render::render_updates(&game, user_side.opponent())?)));
    database.write_game(&game)?;

    Ok(GameResponse {
        command_list: command_list(user_result),
        opponent_response: channel_response,
    })
}

/// Sends a game response to a given player, if they are connected to the
/// server.
pub async fn send_player_response(response: Option<(PlayerId, CommandList)>) {
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
    verify!(!standard_action.payload.is_empty(), "Empty action payload received");
    let action: UserAction = de::from_slice(&standard_action.payload)
        .with_error(|| "Failed to deserialize action payload")?;
    match action {
        UserAction::Debug(debug_action) => {
            debug::handle_debug_action(database, player_id, game_id, debug_action)
        }
        _ => handle_action(database, player_id, game_id, action),
    }
}

/// Look up the state for a game which is expected to exist and assigns an
/// [UpdateTracker] to it for the duration of this request.
pub fn find_game(database: &impl Database, game_id: Option<GameId>) -> Result<GameState> {
    let id = game_id.as_ref().with_error(|| "GameId not provided!")?;
    let mut game = database.game(*id)?;
    game.updates = UpdateTracker::new(if game.data.config.simulation {
        Updates::Ignore
    } else {
        Updates::Push
    });

    Ok(game)
}

/// Turns an `&Option<PlayerIdentifier>` into a [PlayerId], or returns an error
/// if the input is `None`.
pub fn player_id(
    database: &mut impl Database,
    identifier: &Option<PlayerIdentifier>,
) -> Result<PlayerId> {
    database
        .adapt_player_identifier(identifier.as_ref().with_error(|| "Expected player identifier")?)
}

/// Returns the [Side] the indicated user is representing in this game
pub fn user_side(player_id: PlayerId, game: &GameState) -> Result<Side> {
    if player_id == game.champion.id {
        Ok(Side::Champion)
    } else if player_id == game.overlord.id {
        Ok(Side::Overlord)
    } else {
        fail!("User {:?} is not a participant in game {:?}", player_id, game.id)
    }
}

/// Get a display name for a command. Used for debugging.
pub fn command_name(command: &GameCommand) -> &'static str {
    command.command.as_ref().map_or("None", |c| match c {
        Command::Debug(_) => "Debug",
        Command::Delay(_) => "Delay",
        Command::UpdatePanels(_) => "UpdatePanels",
        Command::TogglePanel(_) => "TogglePanel",
        Command::UpdateGameView(_) => "UpdateGameView",
        Command::VisitRoom(_) => "VisitRoom",
        Command::MoveGameObjects(_) => "MoveGameObjects",
        Command::PlaySound(_) => "PlaySound",
        Command::SetMusic(_) => "SetMusic",
        Command::FireProjectile(_) => "FireProjectile",
        Command::PlayEffect(_) => "PlayEffect",
        Command::DisplayGameMessage(_) => "DisplayGameMessage",
        Command::SetGameObjectsEnabled(_) => "SetGameObjectsEnabled",
        Command::DisplayRewards(_) => "DisplayRewards",
        Command::LoadScene(_) => "LoadScene",
        Command::CreateTokenCard(_) => "CreateTokenCard",
    })
}

fn card_target(target: &Option<CardTarget>) -> game_actions::CardTarget {
    target.as_ref().map_or(game_actions::CardTarget::None, |t| {
        t.card_target.as_ref().map_or(game_actions::CardTarget::None, |t2| match t2 {
            card_target::CardTarget::RoomId(room_id) => adapters::room_id(*room_id)
                .map_or(game_actions::CardTarget::None, game_actions::CardTarget::Room),
        })
    })
}

fn command_list(commands: Vec<Command>) -> CommandList {
    CommandList {
        commands: commands.into_iter().map(|c| GameCommand { command: Some(c) }).collect(),
    }
}
