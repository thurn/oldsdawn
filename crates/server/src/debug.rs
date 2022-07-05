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

use adapters;
use anyhow::Result;
use cards::decklists;
use data::agent_definition::AgentData;
use data::game::GameState;
use data::game_actions::DebugAction;
use data::player_data::{CurrentGame, PlayerData};
use data::player_name::{NamedPlayer, PlayerId};
use data::primitives::{DeckId, GameId, Side};
use data::with_error::WithError;
use protos::spelldawn::client_debug_command::DebugCommand;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{
    ClientDebugCommand, CommandList, GameAction, GameCommand, GameIdentifier, LoadSceneCommand,
    NewGameAction, NewGameDebugOptions, SceneLoadMode,
};
use rules::mana;

use crate::database::Database;
use crate::requests;
use crate::requests::GameResponse;

pub fn handle_debug_action(
    database: &mut impl Database,
    player_id: PlayerId,
    game_id: Option<GameId>,
    action: DebugAction,
) -> Result<GameResponse> {
    match action {
        DebugAction::NewGame(side) => {
            const OVERLORD_DECK_ID: DeckId = DeckId { value: 0 };
            const CHAMPION_DECK_ID: DeckId = DeckId { value: 1 };
            write_default_player(database, player_id, None)?;
            Ok(GameResponse {
                command_list: CommandList {
                    commands: vec![GameCommand {
                        command: Some(Command::Debug(ClientDebugCommand {
                            debug_command: Some(DebugCommand::InvokeAction(GameAction {
                                action: Some(Action::NewGame(NewGameAction {
                                    opponent_id: Some(adapters::named_player_identifier(
                                        NamedPlayer::TestCanonicalDeckNoAction,
                                    )?),
                                    deck: Some(adapters::deck_identifier(match side {
                                        Side::Overlord => OVERLORD_DECK_ID,
                                        Side::Champion => CHAMPION_DECK_ID,
                                    })),
                                    debug_options: Some(NewGameDebugOptions {
                                        deterministic: false,
                                        override_game_identifier: Some(GameIdentifier { value: 0 }),
                                    }),
                                })),
                            })),
                        })),
                    }],
                },
                opponent_response: None,
            })
        }
        DebugAction::JoinGame => {
            let mut game = requests::find_game(database, Some(GameId::new(0)))?;
            if matches!(game.overlord.id, PlayerId::Named(_)) {
                game.overlord.id = player_id;
            } else {
                game.champion.id = player_id;
            }
            database.write_game(&game)?;
            write_default_player(database, player_id, Some(CurrentGame::Playing(GameId::new(0))))?;

            Ok(GameResponse::from_commands(vec![Command::LoadScene(LoadSceneCommand {
                scene_name: "Labyrinth".to_string(),
                mode: SceneLoadMode::Single as i32,
            })]))
        }
        DebugAction::FlipViewpoint => {
            requests::handle_custom_action(database, player_id, game_id, |game, user_side| {
                let opponent_id = game.player(user_side.opponent()).id;
                game.player_mut(user_side.opponent()).id = player_id;
                game.player_mut(user_side).id = opponent_id;
                Ok(())
            })
        }
        DebugAction::AddMana(amount) => {
            requests::handle_custom_action(database, player_id, game_id, |game, user_side| {
                mana::gain(game, user_side, amount);
                Ok(())
            })
        }
        DebugAction::AddActionPoints(amount) => {
            requests::handle_custom_action(database, player_id, game_id, |game, user_side| {
                game.player_mut(user_side).actions += amount;
                Ok(())
            })
        }
        DebugAction::AddScore(amount) => {
            requests::handle_custom_action(database, player_id, game_id, |game, user_side| {
                game.player_mut(user_side).score += amount;
                Ok(())
            })
        }
        DebugAction::SaveState(index) => {
            let mut game = load_game(database, game_id)?;
            game.id = GameId::new(u64::MAX - index);
            database.write_game(&game)?;
            Ok(GameResponse::from_commands(vec![]))
        }
        DebugAction::LoadState(index) => {
            let mut game = database.game(GameId::new(u64::MAX - index))?;
            game.id = game_id.with_error(|| "Expected GameId")?;
            database.write_game(&game)?;
            Ok(GameResponse::from_commands(vec![Command::LoadScene(LoadSceneCommand {
                scene_name: "Labyrinth".to_string(),
                mode: SceneLoadMode::Single.into(),
            })]))
        }
        DebugAction::SetAgent(side, state_predictor, agent) => {
            requests::handle_custom_action(database, player_id, game_id, |game, _| {
                game.player_mut(side).agent = Some(AgentData { name: agent, state_predictor });
                Ok(())
            })
        }
    }
}

fn write_default_player(
    database: &mut impl Database,
    player_id: PlayerId,
    current_game: Option<CurrentGame>,
) -> Result<()> {
    database.write_player(&PlayerData {
        id: player_id,
        current_game,
        decks: vec![
            decklists::canonical_deck(player_id, Side::Overlord),
            decklists::canonical_deck(player_id, Side::Champion),
        ],
        collection: HashMap::default(),
    })
}

fn load_game(database: &mut impl Database, game_id: Option<GameId>) -> Result<GameState> {
    database.game(game_id.with_error(|| "GameId is required")?)
}
