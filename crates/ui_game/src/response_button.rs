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

#![allow(dead_code)]

use data::primitives::CardId;
use ui_core::actions::{InterfaceAction, NoAction};
use ui_core::button::{Button, ButtonType};
use ui_core::prelude::*;

#[derive(Debug)]
struct ResponseButton {
    label: String,
    layout: Layout,
    anchor_to: Option<CardId>,
    primary: bool,
    action: Box<dyn InterfaceAction>,
    shift_down: bool,
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
}

impl Component for ResponseButton {
    fn build(self) -> RenderResult {
        Button::new(self.label)
            .button_type(if self.primary { ButtonType::Primary } else { ButtonType::Secondary })
            .action(self.action)
            .layout(
                self.layout
                    .margin(Edge::Horizontal, 16.px())
                    .margin(Edge::Bottom, if self.shift_down { 200.px() } else { 0.px() }),
            )
            .build()
    }
}
