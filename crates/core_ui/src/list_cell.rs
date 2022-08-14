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

use protos::spelldawn::{FlexAlign, FlexJustify, TextAlign};

use crate::button::Button;
use crate::design::{Font, FontColor, FontSize};
use crate::prelude::*;
use crate::style::WidthMode;
use crate::text::Text;

/// Represents a row within a list of items
#[derive(Debug)]
pub struct ListCell {
    text: String,
    layout: Layout,
    right_add_on: Option<Box<dyn Component>>,
}

impl ListCell {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into(), layout: Layout::default(), right_add_on: None }
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn button(mut self, button: Button) -> Self {
        self.right_add_on = Some(Box::new(button));
        self
    }
}

impl Component for ListCell {
    fn build(self) -> RenderResult {
        let mut result = Row::new(format!("ListCell {}", self.text))
            .style(
                self.layout
                    .to_style()
                    .height(104.px())
                    .width(100.pct())
                    .justify_content(FlexJustify::FlexStart)
                    .align_items(FlexAlign::Center)
                    .flex_shrink(0.0)
                    .padding(Edge::All, 32.px()),
            )
            .child(
                Text::new(self.text, FontSize::Headline)
                    .color(FontColor::PrimaryText)
                    .font(Font::PrimaryText)
                    .text_align(TextAlign::MiddleLeft)
                    .width_mode(WidthMode::Flexible),
            );
        result = if let Some(right_add_on) = self.right_add_on {
            result.child(
                Column::new("RightAddOn")
                    .style(Style::new().margin(Edge::Left, 16.px()))
                    .child_boxed(right_add_on),
            )
        } else {
            result
        };
        result.build()
    }
}
