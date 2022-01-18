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

use anyhow::{anyhow, bail, Context, Result};
use dashmap::DashMap;
use data::card_name::CardName;
use data::deck::Deck;
use data::game::{GameConfiguration, GameState};
use data::primitives::{CardId, GameId, PlayerId, RoomId, Side};
use data::prompt::PromptResponse;
use data::updates::UpdateTracker;
use maplit::hashmap;
use once_cell::sync::Lazy;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::spelldawn_server::Spelldawn;
use protos::spelldawn::{
    card_target, CardIdentifier, CardTarget, CommandList, ConnectRequest, GameCommand,
    GameIdentifier, GameRequest, PlayerSide, RoomIdentifier, StandardAction,
};
use rules::actions::PlayCardTarget;
use rules::{actions, raid};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::{error, info, warn, warn_span};

use crate::database::{Database, SledDatabase};

pub mod database;

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
        let player_id = PlayerId::new(message.user_id);
        warn!(?player_id, ?game_id, "received_connection");

        let (tx, rx) = mpsc::channel(4);

        let mut database = SledDatabase;
        match handle_connect(&mut database, player_id, game_id, message.test_mode) {
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
        let mut database = SledDatabase;
        match handle_request(&mut database, request.get_ref()) {
            Ok(response) => {
                if let Some((player_id, commands)) = response.channel_response {
                    if let Some(channel) = CHANNELS.get(&player_id) {
                        if channel.send(Ok(commands)).await.is_err() {
                            // This returns SendError if the client is disconnected, which isn't a
                            // huge problem. Hopefully they will reconnect again in the future.
                            info!(?player_id, "client_is_disconnected");
                        }
                    }
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

/// A response to a given [GameRequest].
///
/// Returned from [handle_request] to support providing updates to different
/// players in a game.
#[derive(Debug, Clone, Default)]
pub struct GameResponse {
    /// Response to send to the user who made the initial game request.
    pub command_list: CommandList,
    /// Response to send to another user, e.g. to update opponent state.
    pub channel_response: Option<(PlayerId, CommandList)>,
}

/// Processes an incoming client request and returns a [GameResponse] describing
/// required updates to send to connected users.
pub fn handle_request(database: &mut impl Database, request: &GameRequest) -> Result<GameResponse> {
    let game_id = to_server_game_id(&request.game_id);
    let player_id = PlayerId::new(request.user_id);
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
        Action::DrawCard(_) => {
            handle_action(database, player_id, game_id, actions::draw_card_action)
        }
        Action::PlayCard(action) => {
            handle_action(database, player_id, game_id, |game, user_side| {
                actions::play_card_action(
                    game,
                    user_side,
                    to_server_card_id(&action.card_id)?,
                    card_target(&action.target),
                )
            })
        }
        Action::GainMana(_) => {
            handle_action(database, player_id, game_id, actions::gain_mana_action)
        }
        Action::InitiateRaid(action) => {
            handle_action(database, player_id, game_id, |game, user_side| {
                raid::initiate_raid_action(
                    game,
                    user_side,
                    to_server_room_id(RoomIdentifier::from_i32(action.room_id))
                        .with_context(|| format!("Invalid room ID: {:?}", action.room_id))?,
                )
            })
        }
        Action::StandardAction(standard_action) => {
            handle_action(database, player_id, game_id, |game, user_side| {
                handle_standard_action(game, user_side, standard_action)
            })
        }
        _ => Ok(GameResponse::default()),
    }?;

    let commands = response.command_list.commands.iter().map(command_name).collect::<Vec<_>>();
    info!(?player_id, ?commands, "sending_response");

    Ok(response)
}

/// Sets up the game state for a game connection request, either connecting to
/// the `game_id` game or creating a new game if `game_id` is not provided. If
/// `test_mode` is true, the new game's ID will be set to 0.
pub fn handle_connect(
    database: &mut impl Database,
    player_id: PlayerId,
    game_id: Option<GameId>,
    test_mode: bool,
) -> Result<CommandList> {
    let game = if let Some(game_id) = game_id {
        database.game(game_id)?
    } else {
        let new_game_id = if test_mode { GameId::new(0) } else { database.generate_game_id()? };
        let mut game = GameState::new_game(
            new_game_id,
            Deck {
                owner_id: PlayerId::new(2),
                identity: CardName::TestOverlordIdentity,
                cards: hashmap! {
                    CardName::DungeonAnnex => 1,
                    CardName::IceDragon => 44,
                },
            },
            Deck {
                owner_id: PlayerId::new(1),
                identity: CardName::TestChampionIdentity,
                cards: hashmap! {
                    CardName::Greataxe => 45,
                },
            },
            GameConfiguration { deterministic: true, ..GameConfiguration::default() },
        );

        if test_mode {
            game.overlord.actions = 4;
        }

        database.write_game(&game)?;
        info!(?new_game_id, "create_new_game");
        game
    };

    let side = user_side(player_id, &game)?;
    Ok(display::connect(&game, side))
}

/// Queries the [GameState] for a game from the [Database] and then invokes the
/// provided `function` to mutate its state. Converts the resulting [GameState]
/// into a series of client updates for both players in the form of a
/// [GameResponse] and then writes the new game state back to the database.
fn handle_action(
    database: &mut impl Database,
    player_id: PlayerId,
    game_id: Option<GameId>,
    function: impl Fn(&mut GameState, Side) -> Result<()>,
) -> Result<GameResponse> {
    let mut game = find_game(database, game_id)?;
    let user_side = user_side(player_id, &game)?;
    function(&mut game, user_side)?;

    let user_result = display::render_updates(&game, user_side);
    let opponent_id = game.player(user_side.opponent()).id;
    let opponent_result = display::render_updates(&game, user_side.opponent());

    database.write_game(&game)?;
    Ok(GameResponse {
        command_list: user_result,
        channel_response: Some((opponent_id, opponent_result)),
    })
}

/// Parses the serialized payload in a [StandardAction] and dispatches to the
/// correct handler.
fn handle_standard_action(
    game: &mut GameState,
    user_side: Side,
    standard_action: &StandardAction,
) -> Result<()> {
    let action: PromptResponse = bincode::deserialize(&standard_action.payload)
        .with_context(|| "Failed to deserialize action payload")?;
    actions::handle_prompt_response(game, user_side, action)
}

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

/// Converts a client [CardIdentifier] into a server [CardId]
pub fn to_server_card_id(card_id: &Option<CardIdentifier>) -> Result<CardId> {
    if let Some(id) = card_id {
        Ok(CardId {
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

/// Get a display name for a command. Used for debugging.
pub fn command_name(command: &GameCommand) -> &'static str {
    command.command.as_ref().map_or("None", |c| match c {
        Command::DebugLog(_) => "DebugLog",
        Command::RunInParallel(_) => "RunInParallel",
        Command::Delay(_) => "Delay",
        Command::RenderInterface(_) => "RenderInterface",
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
