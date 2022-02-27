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

//! The debug panel provides tools for modifying the game state during
//! development. Typically these options should not be available to production
//! users.

use data::actions::{DebugAction, UserAction};
use data::primitives::Side;
use protos::spelldawn::client_debug_command::DebugCommand;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{
    ClientDebugCommand, FlexAlign, FlexJustify, FlexStyle, FlexWrap, Node, PanelAddress,
    TogglePanelCommand,
};
use ui::components::{Button, Row};
use ui::core::{child, node};
use ui::panel::Panel;
use ui::{core, icons};

/// Renders the debug panel
pub fn render() -> Node {
    node(Panel {
        address: PanelAddress::DebugPanel,
        title: Some("Debug Controls".to_string()),
        width: 1024.0,
        height: 512.0,
        content: Row {
            name: "DebugButtons".to_string(),
            style: FlexStyle {
                wrap: FlexWrap::Wrap.into(),
                align_items: FlexAlign::Center.into(),
                justify_content: FlexJustify::Center.into(),
                ..FlexStyle::default()
            },
            children: vec![
                debug_button(
                    "New Game (O)",
                    UserAction::DebugAction(DebugAction::NewGame(Side::Overlord)),
                ),
                debug_button(
                    "New Game (C)",
                    UserAction::DebugAction(DebugAction::NewGame(Side::Champion)),
                ),
                debug_button("Reset", UserAction::DebugAction(DebugAction::ResetGame)),
                debug_button("Fetch UI", UserAction::DebugAction(DebugAction::FetchStandardPanels)),
                client_debug_button(
                    "Show Logs",
                    vec![
                        Command::Debug(ClientDebugCommand {
                            debug_command: Some(DebugCommand::ShowLogs(())),
                        }),
                        Command::TogglePanel(TogglePanelCommand {
                            panel_address: PanelAddress::DebugPanel.into(),
                            open: false,
                        }),
                    ],
                ),
                debug_button(
                    format!("+10{}", icons::MANA),
                    UserAction::DebugAction(DebugAction::AddMana(10)),
                ),
                debug_button(
                    format!("+{}", icons::ACTION),
                    UserAction::DebugAction(DebugAction::AddActionPoints(1)),
                ),
                debug_button("+ Point", UserAction::DebugAction(DebugAction::AddScore(1))),
                debug_button("Turn", UserAction::DebugAction(DebugAction::SwitchTurn)),
                debug_button("Flip View", UserAction::DebugAction(DebugAction::FlipViewpoint)),
                debug_button(
                    format!("{} 1", icons::SAVE),
                    UserAction::DebugAction(DebugAction::SaveState(1)),
                ),
                debug_button(
                    format!("{} 1", icons::RESTORE),
                    UserAction::DebugAction(DebugAction::LoadState(1)),
                ),
                debug_button(
                    format!("{} 2", icons::SAVE),
                    UserAction::DebugAction(DebugAction::SaveState(2)),
                ),
                debug_button(
                    format!("{} 2", icons::RESTORE),
                    UserAction::DebugAction(DebugAction::LoadState(2)),
                ),
                debug_button(
                    format!("{} 3", icons::SAVE),
                    UserAction::DebugAction(DebugAction::SaveState(3)),
                ),
                debug_button(
                    format!("{} 3", icons::RESTORE),
                    UserAction::DebugAction(DebugAction::LoadState(3)),
                ),
            ],
            ..Row::default()
        },
        show_close_button: true,
        ..Panel::default()
    })
}

fn debug_button(label: impl Into<String>, action: UserAction) -> Option<Node> {
    child(Button {
        label: label.into(),
        action: core::action(Some(action), None),
        style: FlexStyle { margin: core::all_px(8.0), ..FlexStyle::default() },
        ..Button::default()
    })
}

fn client_debug_button(label: impl Into<String>, commands: Vec<Command>) -> Option<Node> {
    child(Button {
        label: label.into(),
        action: core::action(None, Some(commands)),
        style: FlexStyle { margin: core::all_px(8.0), ..FlexStyle::default() },
        ..Button::default()
    })
}
