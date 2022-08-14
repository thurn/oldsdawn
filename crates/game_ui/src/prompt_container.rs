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

use core_ui::prelude::*;
use protos::spelldawn::{FlexAlign, FlexJustify, FlexWrap};

#[derive(Debug, Default)]
pub struct PromptContainer {
    children: Vec<Box<dyn Component>>,
}

impl PromptContainer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn child(mut self, child: impl Component + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }

    pub fn children(mut self, children: Vec<Box<dyn Component>>) -> Self {
        self.children.extend(children);
        self
    }
}

impl Component for PromptContainer {
    fn build(self) -> RenderResult {
        Row::new("PromptContainer")
            .style(
                Style::new()
                    .justify_content(FlexJustify::FlexEnd)
                    .align_items(FlexAlign::Center)
                    .flex_grow(1.0)
                    .wrap(FlexWrap::WrapReverse)
                    .margin(Edge::Horizontal, 16.px()),
            )
            .children_boxed(self.children)
            .build()
    }
}
