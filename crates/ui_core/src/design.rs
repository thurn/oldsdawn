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

//! Core design primitives

use protos::spelldawn::{Dimension, FlexColor, FontAddress};

use crate::style::DimensionExt;

const fn color(red: f32, green: f32, blue: f32, alpha: f32) -> FlexColor {
    FlexColor { red, green, blue, alpha }
}

const WHITE: FlexColor = color(1.0, 1.0, 1.0, 1.0);
pub const BLACK_ALPHA_75: FlexColor = color(0.0, 0.0, 0.0, 0.75);

#[derive(Debug, Clone, Copy)]
pub enum BackgroundColor {
    CardInfo,
}

impl From<BackgroundColor> for FlexColor {
    fn from(color: BackgroundColor) -> Self {
        match color {
            BackgroundColor::CardInfo => BLACK_ALPHA_75,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FontColor {
    PrimaryText,
    ButtonLabel,
    PanelTitle,
}

impl From<FontColor> for FlexColor {
    fn from(color: FontColor) -> Self {
        match color {
            FontColor::PrimaryText => WHITE,
            FontColor::ButtonLabel => WHITE,
            FontColor::PanelTitle => WHITE,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FontSize {
    ButtonLabel,
    ButtonIcon,
    PanelTitle,
    PromptContext,
    SupplementalInfo,
}

impl From<FontSize> for Dimension {
    fn from(size: FontSize) -> Self {
        (match size {
            FontSize::ButtonLabel => 32,
            FontSize::ButtonIcon => 48,
            FontSize::PanelTitle => 48,
            FontSize::PromptContext => 48,
            FontSize::SupplementalInfo => 28,
        })
        .px()
        .into()
    }
}

fn roboto() -> FontAddress {
    FontAddress { address: "Fonts/Roboto".to_string() }
}

fn bluu_next() -> FontAddress {
    FontAddress { address: "Fonts/BluuNext-Bold".to_string() }
}

#[derive(Debug, Clone, Copy)]
pub enum Font {
    PrimaryText,
    PanelTitle,
    ButtonLabel,
}

impl From<Font> for FontAddress {
    fn from(font: Font) -> Self {
        match font {
            Font::PrimaryText => roboto(),
            Font::PanelTitle => bluu_next(),
            Font::ButtonLabel => roboto(),
        }
    }
}
