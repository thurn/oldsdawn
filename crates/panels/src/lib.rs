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
use protos::spelldawn::{InterfacePanel, PanelAddress, RenderInterfaceCommand};

/// Primary entry-point for panels. Given a [PanelAddress], creates its UI
/// hierarchy.
pub fn render_panel(address: PanelAddress) -> Result<RenderInterfaceCommand> {
    Ok(RenderInterfaceCommand {
        panels: vec![InterfacePanel {
            address: address.into(),
            node: Some(match address {
                PanelAddress::Unspecified => bail!("Invalid Panel Address"),
                PanelAddress::DebugPanel => debug_panel::render(),
            }),
        }],
        main_controls: None,
    })
}
