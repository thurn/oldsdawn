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
use data::game::GameState;
use data::primitives::Side;
use prompts::WaitingPrompt;
use protos::spelldawn::InterfaceMainControls;
use rules::raid;
use ui::core::Component;

use crate::prompts;

/// Returns a [InterfaceMainControls] to render the interface state for the
/// provided `game`.
pub fn render(game: &GameState, user_side: Side) -> Option<InterfaceMainControls> {
    if let Some(prompt) = render_prompt(game, user_side).expect("todo") {
        Some(prompt)
    } else if render_prompt(game, user_side.opponent()).expect("todo").is_some() {
        // If the opponent has a prompt, display a 'waiting' indicator
        Some(InterfaceMainControls {
            node: Some((WaitingPrompt {}).render()),
            card_anchor_nodes: vec![],
        })
    } else {
        None
    }
}

/// Renders prompt for a player when one is present
fn render_prompt(game: &GameState, side: Side) -> Result<Option<InterfaceMainControls>> {
    Ok(if let Some(prompt) = &game.player(side).card_prompt {
        Some(prompts::action_prompt(game, side, prompt))
    } else if let Some(prompt) = raid::core::current_prompt(game, side)? {
        Some(prompts::action_prompt(game, side, &prompt))
    } else {
        game.player(side)
            .game_prompt
            .as_ref()
            .map(|prompt| prompts::action_prompt(game, side, prompt))
    })
}
