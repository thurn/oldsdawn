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

use protos::spelldawn::FontAddress;

#[derive(Debug, Clone, Copy)]
pub struct Font(&'static str);

impl Font {
    pub fn build(self) -> Option<FontAddress> {
        Some(FontAddress { address: self.0.to_string() })
    }
}

pub const BUTTON_LABEL: Font = ROBOTO;
pub const PROMPT_CONTEXT: Font = ROBOTO;
pub const PANEL_TITLE: Font = BLUU_NEXT;
pub const SUPPLEMENTAL_INFO_TEXT: Font = ROBOTO;

const ROBOTO: Font = Font("Fonts/Roboto");
const BLUU_NEXT: Font = Font("Fonts/BluuNext-Bold");
