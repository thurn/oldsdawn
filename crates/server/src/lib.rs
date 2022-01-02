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
use data::primitives::{CardId, GameId, RoomId, Side, UserId};
use data::updates::UpdateTracker;
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
static CHANNELS: Lazy<DashMap<UserId, Sender<Result<CommandList, Status>>>> =
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
        let user_id = UserId::new(message.user_id);
        warn!(?user_id, ?game_id, "received_connection");

        let (tx, rx) = mpsc::channel(4);

        let mut database = SledDatabase;
        match handle_connect(&mut database, user_id, game_id, message.test_mode) {
            Ok(commands) => {
                if let Err(error) = tx.send(Ok(commands)).await {
                    error!(?user_id, ?game_id, ?error, "Send Error!");
                    return Err(Status::internal(format!("Send Error: {:#}", error)));
                }
            }
            Err(error) => {
                error!(?user_id, ?game_id, ?error, "Connection Error!");
                return Err(Status::internal(format!("Connection Error: {:#}", error)));
            }
        }

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
                if let Some((user_id, commands)) = response.channel_response {
                    if let Some(channel) = CHANNELS.get(&user_id) {
                        if channel.send(Ok(commands)).await.is_err() {
                            // This returns SendError if the client is disconnected, which isn't a
                            // huge problem. Hopefully they will reconnect again in the future.
                            info!(?user_id, "client_is_disconnected");
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
    pub channel_response: Option<(UserId, CommandList)>,
}

/// Processes an incoming client request and returns a [GameResponse] describing
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
        Action::DrawCard(_) => handle_action(database, user_id, game_id, actions::draw_card_action),
        Action::PlayCard(action) => handle_action(database, user_id, game_id, |game, side| {
            actions::play_card_action(
                game,
                side,
                to_server_card_id(&action.card_id)?,
                card_target(&action.target),
            )
        }),
        Action::GainMana(_) => handle_action(database, user_id, game_id, actions::gain_mana_action),
        _ => Ok(GameResponse::default()),
    }?;

    let commands = response.command_list.commands.iter().map(command_name).collect::<Vec<_>>();
    info!(?user_id, ?commands, "sending_response");

    Ok(response)
}

/// Sets up the game state for a game connection request, either connecting to
/// the `game_id` game or creating a new game if `game_id` is not provided. If
/// `test_mode` is true, the new game's ID will be set to 0.
pub fn handle_connect(
    database: &mut impl Database,
    user_id: UserId,
    game_id: Option<GameId>,
    test_mode: bool,
) -> Result<CommandList> {
    let game = if let Some(game_id) = game_id {
        database.game(game_id)?
    } else {
        let new_game_id = if test_mode { GameId::new(0) } else { database.generate_game_id()? };
        let game = GameState::new_game(
            new_game_id,
            Deck {
                owner_id: UserId::new(2),
                identity: CardName::TestOverlordIdentity,
                cards: hashmap! {
                    CardName::DungeonAnnex => 1,
                    CardName::IceDragon => 44,
                },
            },
            Deck {
                owner_id: UserId::new(1),
                identity: CardName::TestChampionIdentity,
                cards: hashmap! {
                    CardName::Greataxe => 1,
                    CardName::ArcaneRecovery => 44,
                },
            },
            GameConfiguration { deterministic: true, ..GameConfiguration::default() },
        );

        database.write_game(&game)?;
        info!(?new_game_id, "create_new_game");
        game
    };

    let side = user_side(user_id, &game)?;
    Ok(display::connect(&game, side))
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

    let user_result = display::render_updates(&game, side);
    let opponent_id = game.player(side.opponent()).id;
    let opponent_result = display::render_updates(&game, side.opponent());

    database.write_game(&game)?;
    Ok(GameResponse {
        command_list: user_result,
        channel_response: Some((opponent_id, opponent_result)),
    })
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
pub fn user_side(user_id: UserId, game: &GameState) -> Result<Side> {
    if user_id == game.champion.id {
        Ok(Side::Champion)
    } else if user_id == game.overlord.id {
        Ok(Side::Overlord)
    } else {
        bail!("User {:?} is not a participant in game {:?}", user_id, game.id)
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
