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

use anyhow::{Context, Result};
use data::actions::DebugAction;
use data::deck::Deck;
use data::delegates::{DawnEvent, DuskEvent};
use data::game::GameState;
use data::primitives::{GameId, PlayerId, Side};
use data::updates::GameUpdate;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{LoadSceneCommand, PanelAddress, SceneLoadMode};
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
        DebugAction::AddMana => {
            crate::handle_action(database, player_id, game_id, |game, user_side| {
                game.player_mut(user_side).mana += 10;
                Ok(())
            })
        }
        DebugAction::AddActionPoints => {
            crate::handle_action(database, player_id, game_id, |game, user_side| {
                game.player_mut(user_side).actions += 1;
                Ok(())
            })
        }
        DebugAction::AddScore => {
            crate::handle_action(database, player_id, game_id, |game, user_side| {
                game.player_mut(user_side).score += 1;
                Ok(())
            })
        }
        DebugAction::SwitchTurn => crate::handle_action(database, player_id, game_id, |game, _| {
            game.player_mut(game.data.turn).actions = 0;
            let new_turn = game.data.turn.opponent();
            game.data.turn = new_turn;
            if new_turn == Side::Overlord {
                game.data.turn_number += 1;
                dispatch::invoke_event(game, DuskEvent(game.data.turn_number));
            } else {
                dispatch::invoke_event(game, DawnEvent(game.data.turn_number));
            }
            game.player_mut(new_turn).actions = queries::start_of_turn_action_count(game, new_turn);
            game.updates.push(GameUpdate::StartTurn(new_turn));
            Ok(())
        }),
    }
}

fn reset_game(database: &mut impl Database, game_id: Option<GameId>) -> Result<()> {
    let id = game_id.with_context(|| "GameId is required")?;
    let game = database.game(id)?;
    database.write_game(&GameState::new_game(
        id,
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
