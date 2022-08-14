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
pub mod panel_address;
pub mod set_player_name_panel;

use anyhow::Result;
use core_ui::{panel, rendering};
use debug_panel::DebugPanel;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::interface_panel_address::AddressType;
use protos::spelldawn::{
    ClientPanelAddress, InterfacePanel, InterfacePanelAddress, Node, UpdatePanelsCommand,
};
use serde_json::de;
use with_error::WithError;

use crate::panel_address::PanelAddress;
use crate::set_player_name_panel::SetPlayerNamePanel;

/// Appends a command to `commands` to render commonly-used panels on connect.
pub fn append_standard_panels(commands: &mut Vec<Command>) -> Result<()> {
    commands
        .push(Command::UpdatePanels(render_panel(panel::client(ClientPanelAddress::DebugPanel))?));
    Ok(())
}

pub fn render_panel(address: InterfacePanelAddress) -> Result<UpdatePanelsCommand> {
    let node = match address.address_type.as_ref().with_error(|| "missing address_type")? {
        AddressType::Serialized(payload) => {
            let address = de::from_slice(payload).with_error(|| "deserialization failed")?;
            render_server_panel(address)
        }
        AddressType::ClientPanel(client_panel) => render_client_panel(
            ClientPanelAddress::from_i32(*client_panel).with_error(|| "invalid known panel")?,
        ),
    };

    Ok(UpdatePanelsCommand { panels: vec![InterfacePanel { address: Some(address), node }] })
}

fn render_server_panel(address: PanelAddress) -> Option<Node> {
    match address {
        PanelAddress::SetPlayerName(side) => rendering::component(SetPlayerNamePanel::new(side)),
    }
}

fn render_client_panel(address: ClientPanelAddress) -> Option<Node> {
    match address {
        ClientPanelAddress::Unspecified => None,
        ClientPanelAddress::DebugPanel => rendering::component(DebugPanel {}),
    }
}
