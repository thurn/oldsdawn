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

use protos::spelldawn::Dimension;

use crate::core::px;

#[derive(Debug, Clone, Copy)]
pub struct FontSize(f32);

impl FontSize {
    pub fn build(self) -> Option<Dimension> {
        px(self.0)
    }
}

pub const PANEL_TITLE: FontSize = FontSize(48.0);
pub const PROMPT_CONTEXT: FontSize = FontSize(48.0);
pub const BUTTON_ICON: FontSize = FontSize(48.0);
pub const BUTTON: FontSize = FontSize(32.0);
pub const SUPPLEMENTAL_INFO: FontSize = FontSize(32.0);
pub const TWO_LINE_BUTTON: FontSize = FontSize(32.0);
pub const SUPPLEMENTAL_INFO_TEXT: FontSize = FontSize(28.0);
