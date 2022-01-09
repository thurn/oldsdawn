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

use data::prompt::{Prompt, PromptKind, PromptResponse, RaidActivateRoom};
use protos::spelldawn::{FlexAlign, FlexJustify, FlexStyle, FlexWrap, Node};

use crate::components::{Button, ButtonVariant, Row, Text, TextVariant};
use crate::core::*;
use crate::macros::children;

/// Component to render a given [Prompt]
#[derive(Debug, Clone)]
pub struct ActionPrompt {
    pub prompt: Prompt,
}

impl Component for ActionPrompt {
    fn render(self) -> Node {
        let mut children = vec![];

        if let Some(label) = prompt_context(self.prompt.kind) {
            children.push(child(Text { label, variant: TextVariant::Title, ..Text::default() }))
        }

        for response in &self.prompt.responses {
            children.push(child(ResponseButton { response: *response }))
        }

        node(PromptContainer { name: "Prompt", children })
    }
}

/// Component to display a waiting message while the opponent is deciding on
/// some action
#[derive(Debug, Clone, Default)]
pub struct WaitingPrompt;

impl Component for WaitingPrompt {
    fn render(self) -> Node {
        node(PromptContainer {
            name: "WaitingPrompt",
            children: children!(Text {
                label: "Waiting for Opponent...".to_string(),
                variant: TextVariant::Title,
                style: FlexStyle { margin: left_right_px(16.0), ..FlexStyle::default() },
                ..Text::default()
            }),
            ..PromptContainer::default()
        })
    }
}

/// Container component for interface prompts
#[derive(Debug, Clone, Default)]
struct PromptContainer {
    pub name: &'static str,
    pub children: Vec<Option<Node>>,
}

impl Component for PromptContainer {
    fn render(self) -> Node {
        node(Row {
            name: self.name.to_string(),
            style: FlexStyle {
                justify_content: FlexJustify::FlexEnd.into(),
                flex_grow: Some(1.0),
                align_items: FlexAlign::Center.into(),
                wrap: FlexWrap::WrapReverse.into(),
                margin: left_right_px(16.0),
                ..FlexStyle::default()
            },
            children: self.children,
            ..Row::default()
        })
    }
}

fn prompt_context(kind: PromptKind) -> Option<String> {
    match kind {
        PromptKind::RaidActivateRoom => Some("Raid:".to_string()),
        _ => None,
    }
}

/// Component for rendering a single prompt response button
#[derive(Debug, Clone)]
struct ResponseButton {
    pub response: PromptResponse,
}

impl Component for ResponseButton {
    fn render(self) -> Node {
        let config = self.config();
        node(Button {
            label: config.label,
            variant: if config.primary { ButtonVariant::Primary } else { ButtonVariant::Secondary },
            style: FlexStyle { margin: left_right_px(16.0), ..FlexStyle::default() },
            ..Button::default()
        })
    }
}

#[derive(Debug, Clone)]
struct ResponseButtonConfig {
    pub label: String,
    pub primary: bool,
}

impl ResponseButtonConfig {
    pub fn new(label: &'static str, primary: bool) -> Self {
        Self { label: label.to_string(), primary }
    }
}

impl ResponseButton {
    fn config(self) -> ResponseButtonConfig {
        match self.response {
            PromptResponse::RaidActivateRoom(activate) => Self::activate_config(activate),
            _ => todo!("Not yet implemented"),
        }
    }

    fn activate_config(activate: RaidActivateRoom) -> ResponseButtonConfig {
        match activate {
            RaidActivateRoom::Activate => ResponseButtonConfig::new("Activate", true),
            RaidActivateRoom::Pass => ResponseButtonConfig::new("Pass", false),
        }
    }
}
