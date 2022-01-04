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

use crate::components::{Button, ButtonVariant, Text, TextVariant};
use crate::macros::children;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{
    render_interface_command, FlexAlign, FlexJustify, FlexStyle, FlexWrap,
    InterfacePositionMainControls, Node, RenderInterfaceCommand,
};

use crate::core::*;

/// Renders a main control prompt, typically displaying a selection of buttons
/// with choices for the user.
pub fn prompt(name: impl Into<String>, children: Vec<Node>) -> Command {
    Command::RenderInterface(RenderInterfaceCommand {
        position: Some(render_interface_command::Position::MainControls(
            InterfacePositionMainControls {
                node: Some(row(
                    name,
                    FlexStyle {
                        justify_content: FlexJustify::FlexEnd.into(),
                        flex_grow: Some(1.0),
                        align_items: FlexAlign::Center.into(),
                        wrap: FlexWrap::WrapReverse.into(),
                        margin: left_right_px(16.0),
                        ..FlexStyle::default()
                    },
                    children,
                )),
            },
        )),
    })
}

pub fn waiting_prompt() -> Command {
    prompt(
        "WaitingPrompt",
        children!(Text {
            label: "Waiting for Opponent...".to_string(),
            variant: TextVariant::Title,
            style: FlexStyle { margin: left_right_px(16.0), ..FlexStyle::default() },
            ..Text::default()
        }),
    )
}

/// Displays a prompt for the Overlord to activate a room
pub fn activation_prompt() -> Command {
    prompt(
        "ActivationPrompt",
        children!(
            Text { label: "Raid:".to_string(), variant: TextVariant::Title, ..Text::default() },
            Button {
                label: "Activate".to_string(),
                variant: ButtonVariant::Primary,
                style: FlexStyle { margin: left_right_px(16.0), ..FlexStyle::default() },
                ..Button::default()
            },
            Button {
                label: "Pass".to_string(),
                variant: ButtonVariant::Secondary,
                ..Button::default()
            }
        ),
    )
}
