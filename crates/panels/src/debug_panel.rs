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

use protos::spelldawn::{Node, PanelAddress};
use ui::components::Button;
use ui::core::node;
use ui::panel::Panel;

/// Renders the debug panel
pub fn render() -> Node {
    node(Panel {
        address: PanelAddress::DebugPanel,
        title: Some("Debug Controls".to_string()),
        width: 1024.0,
        height: 512.0,
        content: Button { label: "Hello, debug".to_string(), ..Button::default() },
        show_close_button: true,
        ..Panel::default()
    })
}
