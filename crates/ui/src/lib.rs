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

//! Library for user interface rendering

use protos::spelldawn::game_command::Command;
use protos::spelldawn::{
    render_interface_command, InterfacePositionMainControls, RenderInterfaceCommand,
};

use crate::core::Component;

pub mod components;
pub mod core;
pub mod macros;
pub mod primitives;
pub mod prompts;

/// Renders a given [Component] as the main interface controls, appearing
/// immediately above the user's hand
pub fn main_controls(component: impl Component) -> Command {
    Command::RenderInterface(RenderInterfaceCommand {
        position: Some(render_interface_command::Position::MainControls(
            InterfacePositionMainControls { node: Some(component.render()) },
        )),
    })
}

/// Command to clear all content in the main interface controls
pub fn clear_main_controls() -> Command {
    Command::RenderInterface(RenderInterfaceCommand {
        position: Some(render_interface_command::Position::MainControls(
            InterfacePositionMainControls { node: None },
        )),
    })
}
