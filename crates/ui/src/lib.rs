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

use protos::spelldawn::{InterfacePositionMainControls, RenderInterfaceCommand};

use crate::core::Component;

pub mod components;
pub mod core;
pub mod macros;
pub mod primitives;
pub mod prompts;

/// Renders a given [Component] as the main interface controls via
/// [RenderInterfaceCommand], appearing immediately above the user's hand
pub fn main_controls(component: impl Component) -> RenderInterfaceCommand {
    RenderInterfaceCommand {
        full_screen: None,
        main_controls: Some(InterfacePositionMainControls { node: Some(component.render()) }),
        card_anchors: None,
    }
}

/// Command to clear all content in the interface
pub fn clear_interface() -> RenderInterfaceCommand {
    RenderInterfaceCommand::default()
}
