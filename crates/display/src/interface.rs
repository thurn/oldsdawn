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

use anyhow::Result;
use core_ui::rendering;
use data::game::{GamePhase, GameState, MulliganDecision};
use data::game_actions::{GamePrompt, PromptAction};
use data::primitives::Side;
use game_ui::prompts;
use game_ui::waiting_prompt::WaitingPrompt;
use protos::spelldawn::InterfaceMainControls;

/// Returns a [InterfaceMainControls] to render the interface state for the
/// provided `game`.
pub fn render(game: &GameState, user_side: Side) -> Result<Option<InterfaceMainControls>> {
    Ok(if let Some(prompt) = render_prompt(game, user_side)? {
        Some(prompt)
    } else if render_prompt(game, user_side.opponent())?.is_some() {
        // If the opponent has a prompt, display a 'waiting' indicator
        Some(InterfaceMainControls {
            node: rendering::component(WaitingPrompt {}),
            card_anchor_nodes: vec![],
        })
    } else {
        None
    })
}

/// Renders prompt for a player when one is present
fn render_prompt(game: &GameState, side: Side) -> Result<Option<InterfaceMainControls>> {
    if let Some(prompt) = &game.player(side).prompt {
        return prompts::action_prompt(game, side, prompt);
    } else if let Some(prompt) = raids::current_prompt(game, side)? {
        return prompts::action_prompt(game, side, &prompt);
    } else if let GamePhase::ResolveMulligans(data) = &game.data.phase {
        if data.decision(side).is_none() {
            return prompts::action_prompt(
                game,
                side,
                &GamePrompt {
                    context: None,
                    responses: vec![
                        PromptAction::MulliganDecision(MulliganDecision::Keep),
                        PromptAction::MulliganDecision(MulliganDecision::Mulligan),
                    ],
                },
            );
        }
    }

    Ok(None)
}
