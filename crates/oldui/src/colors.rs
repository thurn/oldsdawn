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

use protos::spelldawn::FlexColor;

pub type Color = Option<FlexColor>;

const fn color(red: f32, green: f32, blue: f32, alpha: f32) -> Color {
    Some(FlexColor { red, green, blue, alpha })
}

pub const CARD_INFO_BACKGROUND: Color = color(0.0, 0.0, 0.0, 0.75);
pub const PANEL_TITLE: Color = WHITE;
pub const BUTTON_LABEL: Color = WHITE;
pub const PROMPT_CONTEXT: Color = WHITE;
pub const SUPPLEMENTAL_INFO_TEXT: Color = WHITE;
pub const DEBUG_RED: Color = RED;
pub const DEBUG_GREEN: Color = GREEN;
pub const DEBUG_BLUE: Color = BLUE;

const WHITE: Color = color(1.0, 1.0, 1.0, 1.0);
const RED: Color = color(1.0, 0.0, 0.0, 1.0);
const GREEN: Color = color(0.0, 1.0, 0.0, 1.0);
const BLUE: Color = color(0.0, 0.0, 1.0, 1.0);
