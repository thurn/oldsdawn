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

use data::actions::{
    ActivateRoomAction, AdvanceAction, EncounterAction, Prompt, PromptAction, PromptContext,
    UserAction,
};
use data::game::GameState;
use data::primitives::CardId;
use protos::spelldawn::{
    AnchorCorner, CardAnchor, CardAnchorNode, CardNodeAnchorPosition, FlexAlign, FlexJustify,
    FlexStyle, FlexWrap, InterfaceMainControls, Node, RenderInterfaceCommand,
};
use rules::queries;
use ui::components::{Button, ButtonLines, ButtonVariant, Row, Text, TextVariant};
use ui::core::*;
use ui::icons;

use crate::adapters;

pub fn action_prompt(game: &GameState, prompt: &Prompt) -> RenderInterfaceCommand {
    let mut main_controls = vec![];
    let mut card_anchor_nodes = vec![];

    if let Some(label) = prompt_context(prompt.context) {
        main_controls.push(child(Text {
            label,
            variant: TextVariant::PanelTitle,
            ..Text::default()
        }))
    }

    for response in &prompt.responses {
        let button = response_button(game, *response);
        if let Some(anchor_to_card) = button.anchor_to_card {
            card_anchor_nodes.push(CardAnchorNode {
                card_id: Some(adapters::adapt_card_id(anchor_to_card)),
                node: Some(
                    Row {
                        style: FlexStyle {
                            padding: top_px(8.0),
                            justify_content: FlexJustify::Center.into(),
                            align_items: FlexAlign::Center.into(),
                            ..FlexStyle::default()
                        },
                        children: vec![button.child()],
                        ..Row::default()
                    }
                    .render(),
                ),
                anchor_position: CardNodeAnchorPosition::Unspecified.into(),
                anchors: vec![
                    CardAnchor {
                        node_corner: AnchorCorner::TopLeft.into(),
                        card_corner: AnchorCorner::BottomLeft.into(),
                    },
                    CardAnchor {
                        node_corner: AnchorCorner::TopRight.into(),
                        card_corner: AnchorCorner::BottomRight.into(),
                    },
                ],
            });
        } else {
            main_controls.push(child(button));
        }
    }

    RenderInterfaceCommand {
        panels: vec![],
        main_controls: Some(InterfaceMainControls {
            node: Some(node(PromptContainer { name: "Prompt", children: main_controls })),
            card_anchor_nodes,
        }),
    }
}

/// Component to render a given [Prompt]
#[derive(Debug, Clone)]
pub struct ActionPrompt<'a> {
    pub game: &'a GameState,
    pub prompt: &'a Prompt,
}

impl<'a> Component for ActionPrompt<'a> {
    fn render(self) -> Node {
        let mut children = vec![];

        if let Some(label) = prompt_context(self.prompt.context) {
            children.push(child(Text {
                label,
                variant: TextVariant::PanelTitle,
                ..Text::default()
            }))
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
            children: vec![Text {
                label: "Waiting for Opponent...".to_string(),
                variant: TextVariant::PanelTitle,
                style: FlexStyle { margin: px_pair(0.0, 16.0), ..FlexStyle::default() },
                ..Text::default()
            }
            .child()],
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
                margin: px_pair(0.0, 16.0),
                ..FlexStyle::default()
            },
            children: self.children,
            ..Row::default()
        })
    }
}

fn prompt_context(context: Option<PromptContext>) -> Option<String> {
    context.map(|context| match context {
        PromptContext::ActivateRoom => "Raid:".to_string(),
        PromptContext::RaidAdvance => "Continue?".to_string(),
    })
}

fn response_button(game: &GameState, response: PromptAction) -> ResponseButton {
    let button = match response {
        PromptAction::ActivateRoomAction(activate) => activate_button(activate),
        PromptAction::EncounterAction(encounter_action) => {
            encounter_action_button(game, encounter_action)
        }
        PromptAction::AdvanceAction(advance_action) => advance_action_button(advance_action),
        PromptAction::RaidScoreCard(card_id) => ResponseButton {
            label: "Score!".to_string(),
            anchor_to_card: Some(card_id),
            ..ResponseButton::default()
        },
        PromptAction::RaidDestroyCard(card_id) => {
            let cost = queries::shield(game, card_id);
            ResponseButton {
                label: if cost == 0 {
                    "Raze".to_string()
                } else {
                    format!("{}{}: Raze", cost, icons::MANA)
                },
                anchor_to_card: Some(card_id),
                ..ResponseButton::default()
            }
        }
        PromptAction::EndRaid => ResponseButton {
            label: "End Raid".to_string(),
            primary: false,
            shift_down: true,
            ..ResponseButton::default()
        },
    };
    ResponseButton { action: Some(response), ..button }
}

/// Component for rendering a single prompt response button
#[derive(Debug, Clone)]
struct ResponseButton {
    pub label: String,
    pub anchor_to_card: Option<CardId>,
    pub primary: bool,
    pub two_lines: bool,
    pub action: Option<PromptAction>,
    pub shift_down: bool,
}

impl Default for ResponseButton {
    fn default() -> Self {
        Self {
            label: "".to_string(),
            anchor_to_card: None,
            primary: true,
            two_lines: false,
            action: None,
            shift_down: false,
        }
    }
}

impl Component for ResponseButton {
    fn render(self) -> Node {
        node(Button {
            label: self.label,
            variant: if self.primary { ButtonVariant::Primary } else { ButtonVariant::Secondary },
            action: self.action.and_then(|a| {
                action(Some(UserAction::PromptAction(a)), ui::clear_main_controls_command())
            }),
            lines: if self.two_lines { ButtonLines::TwoLines } else { ButtonLines::OneLine },
            style: FlexStyle {
                margin: dimension_group_px(
                    0.0,
                    16.0,
                    if self.shift_down { 200.0 } else { 0.0 },
                    16.0,
                ),
                ..FlexStyle::default()
            },
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
            if let Some(cost) = queries::cost_to_defeat_target(game, source_id, target_id) {
                ResponseButton {
                    label: if cost > 0 {
                        format!("{}\n{}{}", label, cost, icons::MANA)
                    } else {
                        label
                    },
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

fn advance_action_button(advance_action: AdvanceAction) -> ResponseButton {
    match advance_action {
        AdvanceAction::Advance => ResponseButton {
            label: "Advance".to_string(),
            primary: true,
            ..ResponseButton::default()
        },
        AdvanceAction::Retreat => ResponseButton {
            label: "Continue".to_string(),
            primary: false,
            ..ResponseButton::default()
        },
    }
}
