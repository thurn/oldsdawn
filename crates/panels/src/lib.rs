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

//! Panel rendering. A 'panel' is a discrete rectangular piece of UI which can
//! be opened or closed by the user, such as a game menu or window.

pub mod debug_panel;

use anyhow::{bail, Result};
use data::fail;
use data::with_error::WithError;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::panel_address::AddressType;
use protos::spelldawn::{InterfacePanel, KnownPanelAddress, PanelAddress, UpdatePanelsCommand};
use ui::core;

/// Appends a command to `commands` to render commonly-used panels.
pub fn render_standard_panels(commands: &mut Vec<Command>) -> Result<()> {
    commands.push(Command::UpdatePanels(render_known_panel(KnownPanelAddress::DebugPanel)?));
    Ok(())
}

pub fn render_panel(address: &PanelAddress) -> Result<UpdatePanelsCommand> {
    match address.address_type.as_ref().with_error(|| "missing address_type")? {
        AddressType::Serialized(_) => fail!("Not yet implemented"),
        AddressType::KnownPanel(known_panel) => render_known_panel(
            KnownPanelAddress::from_i32(*known_panel).with_error(|| "invalid known panel")?,
        ),
    }
}

/// Primary entry-point for panels. Given a [KnownPanelAddress], creates its UI
/// hierarchy.
pub fn render_known_panel(address: KnownPanelAddress) -> Result<UpdatePanelsCommand> {
    Ok(UpdatePanelsCommand {
        panels: vec![InterfacePanel {
            address: Some(core::known_address(address)),
            node: Some(match address {
                KnownPanelAddress::Unspecified => fail!("Invalid Panel Address"),
                KnownPanelAddress::DebugPanel => debug_panel::render(),
            }),
        }],
    })
}
