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

//! Contains primitive design elements such as colors & fonts. Intended to be
//! used via wildcard import in UI definition files.

use protos::spelldawn::{FlexColor, FontAddress};

pub const WHITE: Option<FlexColor> =
    Some(FlexColor { red: 1.0, green: 1.0, blue: 1.0, alpha: 1.0 });
pub const BLACK: Option<FlexColor> =
    Some(FlexColor { red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0 });
pub const GRAY: Option<FlexColor> = Some(FlexColor { red: 0.5, green: 0.5, blue: 0.5, alpha: 1.0 });
pub const RED: Option<FlexColor> = Some(FlexColor { red: 1.0, green: 0.0, blue: 0.0, alpha: 1.0 });
pub const GREEN: Option<FlexColor> =
    Some(FlexColor { red: 0.0, green: 1.0, blue: 0.0, alpha: 1.0 });
pub const BLUE: Option<FlexColor> = Some(FlexColor { red: 0.0, green: 0.0, blue: 1.0, alpha: 1.0 });
pub const MAGENTA: Option<FlexColor> =
    Some(FlexColor { red: 1.0, green: 0.0, blue: 1.0, alpha: 1.0 });
pub const CYAN: Option<FlexColor> = Some(FlexColor { red: 0.0, green: 1.0, blue: 1.0, alpha: 1.0 });
pub const YELLOW: Option<FlexColor> =
    Some(FlexColor { red: 1.0, green: 1.0, blue: 0.0, alpha: 1.0 });

/// Possible interface colors
#[derive(Debug, Clone, Copy)]
pub enum Color {
    TitleText,
    ButtonLabel,
}

/// Returns the [FlexColor] to use for a given interface color. Prefer using
/// this function to directly accessing the color constants in this module.
pub fn color(color: Color) -> Option<FlexColor> {
    match color {
        Color::TitleText => WHITE,
        Color::ButtonLabel => WHITE,
    }
}

/// Possible interface fonts
#[derive(Debug, Clone, Copy)]
pub enum Font {
    Default,
}

pub fn font(font: Font) -> Option<FontAddress> {
    Some(FontAddress {
        address: match font {
            Font::Default => "Fonts/Roboto",
        }
        .to_string(),
    })
}
