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

use anyhow::{bail, Context, Result};
use data::actions::DebugAction;
use data::deck::Deck;
use data::delegates::{DawnEvent, DuskEvent};
use data::game::GameState;
use data::primitives::{GameId, PlayerId, Side};
use data::updates::GameUpdate;
use display::adapters;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{
    LoadSceneCommand, PanelAddress, PlayerIdentifier, SceneLoadMode, SetPlayerIdentifierCommand,
};
use rules::{dispatch, queries};

use crate::{Database, GameResponse};

pub fn handle_debug_action(
    database: &mut impl Database,
    player_id: PlayerId,
    game_id: Option<GameId>,
    action: DebugAction,
) -> Result<GameResponse> {
    match action {
        DebugAction::ResetGame => {
            reset_game(database, game_id)?;
            display::on_disconnect(player_id);
            Ok(GameResponse::from_commands(vec![Command::LoadScene(LoadSceneCommand {
                scene_name: "Labyrinth".to_string(),
                mode: SceneLoadMode::Single.into(),
            })]))
        }
        DebugAction::FetchStandardPanels => {
            Ok(GameResponse::from_commands(vec![Command::RenderInterface(panels::render_panel(
                PanelAddress::DebugPanel,
            )?)]))
        }
        DebugAction::AddMana(amount) => {
            crate::handle_action(database, player_id, game_id, |game, user_side| {
                game.player_mut(user_side).mana += amount;
                Ok(())
            })
        }
        DebugAction::AddActionPoints(amount) => {
            crate::handle_action(database, player_id, game_id, |game, user_side| {
                game.player_mut(user_side).actions += amount;
                Ok(())
            })
        }
        DebugAction::AddScore(amount) => {
            crate::handle_action(database, player_id, game_id, |game, user_side| {
                game.player_mut(user_side).score += amount;
                Ok(())
            })
        }
        DebugAction::SwitchTurn => crate::handle_action(database, player_id, game_id, |game, _| {
            game.player_mut(game.current_turn()?.side).actions = 0;
            let new_turn = game.current_turn()?.side.opponent();
            game.current_turn_mut()?.side = new_turn;
            if new_turn == Side::Overlord {
                game.current_turn_mut()?.turn_number += 1;
                dispatch::invoke_event(game, DuskEvent(game.current_turn()?.turn_number));
            } else {
                dispatch::invoke_event(game, DawnEvent(game.current_turn()?.turn_number));
            }
            game.player_mut(new_turn).actions = queries::start_of_turn_action_count(game, new_turn);
            game.updates.push(GameUpdate::StartTurn(new_turn));
            Ok(())
        }),
        DebugAction::FlipViewpoint => {
            display::on_disconnect(player_id);
            Ok(GameResponse::from_commands(vec![
                Command::SetPlayerId(SetPlayerIdentifierCommand {
                    id: Some(flipped_viewpoint(database, player_id, game_id)?),
                }),
                Command::LoadScene(LoadSceneCommand {
                    scene_name: "Labyrinth".to_string(),
                    mode: SceneLoadMode::Single.into(),
                }),
            ]))
        }
        DebugAction::SaveState(index) => {
            let mut game = load_game(database, game_id)?;
            game.id = GameId::new(u64::MAX - index);
            database.write_game(&game)?;
            Ok(GameResponse::from_commands(vec![]))
        }
        DebugAction::LoadState(index) => {
            let mut game = database.game(GameId::new(u64::MAX - index))?;
            game.id = game_id.with_context(|| "Expected GameId")?;
            database.write_game(&game)?;
            display::on_disconnect(player_id);
            Ok(GameResponse::from_commands(vec![Command::LoadScene(LoadSceneCommand {
                scene_name: "Labyrinth".to_string(),
                mode: SceneLoadMode::Single.into(),
            })]))
        }
    }
}

fn reset_game(database: &mut impl Database, game_id: Option<GameId>) -> Result<()> {
    let game = load_game(database, game_id)?;
    database.write_game(&GameState::new_game(
        game.id,
        Deck {
            owner_id: game.overlord.id,
            identity: game.identity(Side::Overlord).name,
            cards: game.overlord_cards.iter().fold(HashMap::new(), |mut acc, card| {
                *acc.entry(card.name).or_insert(0) += 1;
                acc
            }),
        },
        Deck {
            owner_id: game.champion.id,
            identity: game.identity(Side::Champion).name,
            cards: game.champion_cards.iter().fold(HashMap::new(), |mut acc, card| {
                *acc.entry(card.name).or_insert(0) += 1;
                acc
            }),
        },
        game.data.config,
    ))?;
    Ok(())
}

fn flipped_viewpoint(
    database: &mut impl Database,
    player_id: PlayerId,
    game_id: Option<GameId>,
) -> Result<PlayerIdentifier> {
    let game = load_game(database, game_id)?;
    if player_id == game.overlord.id {
        Ok(adapters::adapt_player_id(game.champion.id))
    } else if player_id == game.champion.id {
        Ok(adapters::adapt_player_id(game.overlord.id))
    } else {
        bail!("ID must be present in game")
    }
}

fn load_game(database: &mut impl Database, game_id: Option<GameId>) -> Result<GameState> {
    database.game(game_id.with_context(|| "GameId is required")?)
}
