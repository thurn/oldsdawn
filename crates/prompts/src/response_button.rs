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

use adapters;
use anyhow::Result;
use core_ui::actions::{InterfaceAction, NoAction};
use core_ui::button::{Button, ButtonType};
use core_ui::prelude::*;
use core_ui::rendering;
use data::primitives::CardId;
use protos::spelldawn::{AnchorCorner, CardAnchor, CardAnchorNode, FlexAlign, FlexJustify};
use with_error::WithError;

#[derive(Debug)]
pub struct ResponseButton {
    label: String,
    layout: Layout,
    anchor_to: Option<CardId>,
    primary: bool,
    action: Box<dyn InterfaceAction>,
    shift_down: bool,
    two_lines: bool,
}

impl ResponseButton {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            layout: Layout::default(),
            anchor_to: None,
            primary: true,
            action: Box::new(NoAction {}),
            shift_down: false,
            two_lines: false,
        }
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn anchor_to(mut self, anchor_to: CardId) -> Self {
        self.anchor_to = Some(anchor_to);
        self
    }

    /// Returns true if this button has been given a [CardId] to anchor to via
    /// [Self::anchor_to] and thus should be rendered using
    /// [Self::render_to_card_anchor_node].
    pub fn has_anchor(&self) -> bool {
        self.anchor_to.is_some()
    }

    pub fn primary(mut self, primary: bool) -> Self {
        self.primary = primary;
        self
    }

    pub fn action(mut self, action: impl InterfaceAction + 'static) -> Self {
        self.action = Box::new(action);
        self
    }

    pub fn shift_down(mut self, shift_down: bool) -> Self {
        self.shift_down = shift_down;
        self
    }

    pub fn two_lines(mut self, is_two_lines: bool) -> Self {
        self.two_lines = is_two_lines;
        self
    }

    pub fn render_to_card_anchor_node(self) -> Result<CardAnchorNode> {
        Ok(CardAnchorNode {
            card_id: Some(adapters::card_identifier(
                self.anchor_to.with_error(|| "Anchor not found")?,
            )),
            node: rendering::component(
                Row::new("CardAnchorButton")
                    .style(
                        Style::new()
                            .padding(Edge::Top, 8.px())
                            .justify_content(FlexJustify::Center)
                            .align_items(FlexAlign::Center),
                    )
                    .child(self),
            ),
            anchors: vec![
                CardAnchor {
                    node_corner: AnchorCorner::TopLeft as i32,
                    card_corner: AnchorCorner::BottomLeft as i32,
                },
                CardAnchor {
                    node_corner: AnchorCorner::TopRight as i32,
                    card_corner: AnchorCorner::BottomRight as i32,
                },
            ],
        })
    }
}

impl Component for ResponseButton {
    fn build(self) -> RenderResult {
        Button::new(self.label)
            .button_type(if self.primary { ButtonType::Primary } else { ButtonType::Secondary })
            .action(self.action)
            .two_lines(self.two_lines)
            .layout(
                self.layout
                    .margin(Edge::Horizontal, 16.px())
                    .margin(Edge::Bottom, if self.shift_down { 200.px() } else { 0.px() }),
            )
            .build()
    }
}
