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
use protos::spelldawn::RenderInterfaceCommand;

use crate::prompts;

/// Returns a [RenderInterfaceCommand] to render the interface state for the
/// provided `game`.
pub fn render(game: &GameState, user_side: Side) -> RenderInterfaceCommand {
    if game.overlord.prompt.is_some() || game.champion.prompt.is_some() {
        render_prompt(game, user_side)
    } else {
        ui::clear_main_controls()
    }
}

/// Renders prompts for both players when one is present
fn render_prompt(game: &GameState, user_side: Side) -> RenderInterfaceCommand {
    game.player(user_side).prompt.as_ref().map_or_else(
        || ui::main_controls(WaitingPrompt {}),
        |prompt| prompts::action_prompt(game, prompt),
    )
}
