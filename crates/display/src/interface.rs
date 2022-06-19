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

use data::game::GameState;
use data::primitives::Side;
use prompts::WaitingPrompt;
use protos::spelldawn::InterfaceMainControls;
use ui::core::Component;

use crate::prompts;

/// Returns a [RenderInterfaceCommand] to render the interface state for the
/// provided `game`.
pub fn render(game: &GameState, user_side: Side) -> Option<InterfaceMainControls> {
    if game.overlord.game_prompt.is_some() || game.champion.game_prompt.is_some() {
        Some(render_prompt(game, user_side))
    } else {
        None
    }
}

/// Renders prompt for a player when one is present
fn render_prompt(game: &GameState, side: Side) -> InterfaceMainControls {
    if let Some(prompt) = &game.player(side).card_prompt {
        prompts::action_prompt(game, side, prompt)
    } else if let Some(prompt) = &game.player(side).game_prompt {
        prompts::action_prompt(game, side, prompt)
    } else {
        InterfaceMainControls { node: Some((WaitingPrompt {}).render()), card_anchor_nodes: vec![] }
    }
}
