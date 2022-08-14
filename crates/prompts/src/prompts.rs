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
use core_ui::design::FontSize;
use core_ui::prelude::Component;
use core_ui::rendering;
use core_ui::text::Text;
use data::game::GameState;
use data::game_actions::{GamePrompt, PromptContext};
use data::primitives::Side;
use protos::spelldawn::InterfaceMainControls;

use crate::action_buttons;
use crate::prompt_container::PromptContainer;

/// Builds UI elements to display a [GamePrompt] for the `side` player.
pub fn action_prompt(
    game: &GameState,
    side: Side,
    prompt: &GamePrompt,
) -> Result<Option<InterfaceMainControls>> {
    let mut main_controls: Vec<Box<dyn Component>> = vec![];
    let mut card_anchor_nodes = vec![];

    if let Some(label) = prompt_context(prompt.context) {
        main_controls.push(Box::new(Text::new(label, FontSize::PromptContext)));
    }

    for response in &prompt.responses {
        let button = action_buttons::for_prompt(game, side, *response);
        if button.has_anchor() {
            card_anchor_nodes.push(button.render_to_card_anchor_node()?);
        } else {
            main_controls.push(Box::new(button));
        }
    }

    Ok(Some(InterfaceMainControls {
        node: rendering::component(PromptContainer::new().children(main_controls)),
        card_anchor_nodes,
    }))
}

fn prompt_context(context: Option<PromptContext>) -> Option<String> {
    context.map(|context| match context {
        PromptContext::RaidAdvance => "Continue?".to_string(),
    })
}
