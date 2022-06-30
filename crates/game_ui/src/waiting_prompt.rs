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

use core_ui::design::FontSize;
use core_ui::prelude::*;
use core_ui::text::Text;

use crate::prompt_container::PromptContainer;

#[derive(Debug)]
pub struct WaitingPrompt;

impl Component for WaitingPrompt {
    fn build(self) -> RenderResult {
        PromptContainer::new()
            .child(
                Text::new("Waiting for Opponent...", FontSize::PromptContext)
                    .layout(Layout::new().margin(Edge::Horizontal, 16.px())),
            )
            .build()
    }
}
