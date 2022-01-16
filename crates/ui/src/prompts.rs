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
use data::prompt::{ActivateRoomAction, EncounterAction, Prompt, PromptKind, PromptResponse};
use protos::spelldawn::{FlexAlign, FlexJustify, FlexStyle, FlexWrap, Node};
use rules::queries;

use crate::components::{Button, ButtonLines, ButtonVariant, Row, Text, TextVariant};
use crate::core::*;
use crate::macros::children;

/// Component to render a given [Prompt]
#[derive(Debug, Clone)]
pub struct ActionPrompt<'a> {
    pub game: &'a GameState,
    pub prompt: Prompt,
}

impl<'a> Component for ActionPrompt<'a> {
    fn render(self) -> Node {
        let mut children = vec![];

        if let Some(label) = prompt_context(self.prompt.kind) {
            children.push(child(Text { label, variant: TextVariant::Title, ..Text::default() }))
        }

        for response in &self.prompt.responses {
            children.push(child(response_button(self.game, *response)))
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
        PromptKind::ActivateRoomAction => Some("Raid:".to_string()),
        _ => None,
    }
}

fn response_button(game: &GameState, response: PromptResponse) -> ResponseButton {
    let button = match response {
        PromptResponse::ActivateRoomAction(activate) => activate_button(activate),
        PromptResponse::EncounterAction(encounter_action) => {
            encounter_action_button(game, encounter_action)
        }
        _ => todo!("Not yet implemented"),
    };
    ResponseButton { action: Some(response), ..button }
}

/// Component for rendering a single prompt response button
#[derive(Debug, Clone)]
struct ResponseButton {
    pub label: String,
    pub primary: bool,
    pub two_lines: bool,
    pub action: Option<PromptResponse>,
}

impl Default for ResponseButton {
    fn default() -> Self {
        Self { label: "".to_string(), primary: true, two_lines: false, action: None }
    }
}

impl Component for ResponseButton {
    fn render(self) -> Node {
        node(Button {
            label: self.label,
            variant: if self.primary { ButtonVariant::Primary } else { ButtonVariant::Secondary },
            action: action(self.action, None),
            lines: if self.two_lines { ButtonLines::TwoLines } else { ButtonLines::OneLine },
            style: FlexStyle { margin: left_right_px(16.0), ..FlexStyle::default() },
            ..Button::default()
        })
    }
}

fn activate_button(activate: ActivateRoomAction) -> ResponseButton {
    match activate {
        ActivateRoomAction::Activate => {
            ResponseButton { label: "Activate".to_string(), ..ResponseButton::default() }
        }
        ActivateRoomAction::Pass => ResponseButton {
            label: "Pass".to_string(),
            primary: false,
            ..ResponseButton::default()
        },
    }
}

fn encounter_action_button(game: &GameState, encounter_action: EncounterAction) -> ResponseButton {
    match encounter_action {
        EncounterAction::UseWeaponAbility(source_id, target_id) => {
            let label = rules::card_definition(game, source_id).name.displayed_name();
            if let Some(cost) =
                queries::boost_target_mana_cost(game, source_id, queries::health(game, target_id))
            {
                ResponseButton {
                    label: if cost > 0 { format!("{}\n{}\u{f06d}", label, cost) } else { label },
                    two_lines: true,
                    ..ResponseButton::default()
                }
            } else {
                ResponseButton { label, ..ResponseButton::default() }
            }
        }
        EncounterAction::Continue => ResponseButton {
            label: "Continue".to_string(),
            primary: false,
            ..ResponseButton::default()
        },
    }
}
