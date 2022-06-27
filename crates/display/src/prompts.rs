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

use data::game::{GameState, MulliganDecision};
use data::game_actions::{
    AccessPhaseAction, CardPromptAction, ContinueAction, EncounterAction, GamePrompt, PromptAction,
    PromptContext, RoomActivationAction, UserAction,
};
use data::primitives::{CardId, Side};
use protos::spelldawn::{
    AnchorCorner, CardAnchor, CardAnchorNode, CardNodeAnchorPosition, FlexAlign, FlexJustify,
    FlexStyle, FlexWrap, InterfaceMainControls, Node,
};
use rules::queries;
use oldui::components::{Button, ButtonLines, ButtonVariant, Row, Text};
use oldui::core::*;
use oldui::{colors, font_sizes, fonts, icons};

use crate::adapters;

/// Command to renders UI elements to display the provided [GamePrompt] for the
/// `side` player.
pub fn action_prompt(
    game: &GameState,
    side: Side,
    prompt: &GamePrompt,
) -> Option<InterfaceMainControls> {
    let mut main_controls = vec![];
    let mut card_anchor_nodes = vec![];

    if let Some(label) = prompt_context(prompt.context) {
        main_controls.push(child(Text {
            label,
            color: colors::PROMPT_CONTEXT,
            font_size: font_sizes::PROMPT_CONTEXT,
            font: fonts::PROMPT_CONTEXT,
            style: FlexStyle::default(),
        }))
    }

    for response in &prompt.responses {
        let button = response_button(game, side, *response);
        if let Some(anchor_to_card) = button.anchor_to_card {
            card_anchor_nodes.push(CardAnchorNode {
                card_id: Some(adapters::card_identifier(anchor_to_card)),
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

    Some(InterfaceMainControls {
        node: Some(node(PromptContainer { name: "Prompt", children: main_controls })),
        card_anchor_nodes,
    })
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
                color: colors::PROMPT_CONTEXT,
                font_size: font_sizes::PROMPT_CONTEXT,
                font: fonts::PROMPT_CONTEXT,
                style: FlexStyle { margin: px_pair(0.0, 16.0), ..FlexStyle::default() },
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
            action: self.action.and_then(|a| action(Some(UserAction::GamePromptResponse(a)), None)),
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

fn response_button(game: &GameState, side: Side, response: PromptAction) -> ResponseButton {
    let button = match response {
        PromptAction::MulliganDecision(mulligan) => mulligan_button(mulligan),
        PromptAction::ActivateRoomAction(activate) => activate_button(activate),
        PromptAction::EncounterAction(encounter_action) => {
            encounter_action_button(game, side, encounter_action)
        }
        PromptAction::ContinueAction(advance_action) => advance_action_button(advance_action),
        PromptAction::AccessPhaseAction(action) => match action {
            AccessPhaseAction::ScoreCard(card_id) => ResponseButton {
                label: "Score!".to_string(),
                anchor_to_card: Some(card_id),
                ..ResponseButton::default()
            },
            AccessPhaseAction::EndRaid => ResponseButton {
                label: "End Raid".to_string(),
                primary: false,
                shift_down: true,
                ..ResponseButton::default()
            },
        },
        PromptAction::CardAction(action) => card_response_button(side, action),
    };
    ResponseButton { action: Some(response), ..button }
}

fn card_response_button(user_side: Side, action: CardPromptAction) -> ResponseButton {
    fn lose_text(user_side: Side, target_side: Side) -> &'static str {
        if user_side == target_side {
            "Pay"
        } else {
            "Lose"
        }
    }

    match action {
        CardPromptAction::LoseMana(side, amount) => ResponseButton {
            label: format!("{} {}{}", lose_text(user_side, side), amount, icons::MANA),
            ..ResponseButton::default()
        },
        CardPromptAction::LoseActions(side, amount) => ResponseButton {
            label: if amount > 1 {
                format!("{} {}{}", lose_text(user_side, side), amount, icons::ACTION)
            } else {
                format!("{} {}", lose_text(user_side, side), icons::ACTION)
            },
            ..ResponseButton::default()
        },
        CardPromptAction::EndRaid => {
            ResponseButton { label: "End Raid".to_string(), ..ResponseButton::default() }
        }
        CardPromptAction::TakeDamage(_, _, amount) => {
            ResponseButton { label: format!("Take {}", amount), ..ResponseButton::default() }
        }
        CardPromptAction::TakeDamageEndRaid(_, _, amount) => ResponseButton {
            label: format!("End Raid, Take {}", amount),
            ..ResponseButton::default()
        },
    }
}

fn mulligan_button(mulligan: MulliganDecision) -> ResponseButton {
    match mulligan {
        MulliganDecision::Keep => {
            ResponseButton { label: "Keep".to_string(), ..ResponseButton::default() }
        }
        MulliganDecision::Mulligan => ResponseButton {
            label: "Mulligan".to_string(),
            primary: false,
            ..ResponseButton::default()
        },
    }
}

fn activate_button(activate: RoomActivationAction) -> ResponseButton {
    match activate {
        RoomActivationAction::Activate => {
            ResponseButton { label: "Activate".to_string(), ..ResponseButton::default() }
        }
        RoomActivationAction::Pass => ResponseButton {
            label: "Pass".to_string(),
            primary: false,
            ..ResponseButton::default()
        },
    }
}

fn encounter_action_button(
    game: &GameState,
    side: Side,
    encounter_action: EncounterAction,
) -> ResponseButton {
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
        EncounterAction::NoWeapon => ResponseButton {
            label: "Continue".to_string(),
            primary: false,
            ..ResponseButton::default()
        },
        EncounterAction::CardAction(card_action) => card_response_button(side, card_action),
    }
}

fn advance_action_button(advance_action: ContinueAction) -> ResponseButton {
    match advance_action {
        ContinueAction::Advance => ResponseButton {
            label: "Advance".to_string(),
            primary: true,
            ..ResponseButton::default()
        },
        ContinueAction::Retreat => ResponseButton {
            label: "Retreat".to_string(),
            primary: false,
            ..ResponseButton::default()
        },
    }
}
