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

pub const WHITE: FlexColor = color(1.0, 1.0, 1.0, 1.0);
pub const BLACK: FlexColor = color(0.0, 0.0, 0.0, 1.0);
pub const BLACK_ALPHA_75: FlexColor = color(0.0, 0.0, 0.0, 0.75);
pub const RED_100: FlexColor = color(1.0, 0.8, 0.82, 1.0);
pub const RED_500: FlexColor = color(0.96, 0.26, 0.21, 1.0);
pub const RED_600: FlexColor = color(0.9, 0.22, 0.21, 1.0);
pub const RED_700: FlexColor = color(0.83, 0.18, 0.18, 1.0);
pub const RED_800: FlexColor = color(0.78, 0.16, 0.16, 1.0);
pub const RED_900: FlexColor = color(0.72, 0.11, 0.11, 1.0);
pub const BLUE_500: FlexColor = color(0.13, 0.59, 0.95, 1.0);
pub const BLUE_700: FlexColor = color(0.1, 0.46, 0.82, 1.0);
pub const BLUE_900: FlexColor = color(0.05, 0.28, 0.63, 1.0);
pub const GREEN_500: FlexColor = color(0.3, 0.69, 0.31, 1.0);
pub const GREEN_700: FlexColor = color(0.22, 0.56, 0.24, 1.0);
pub const GREEN_900: FlexColor = color(0.11, 0.37, 0.13, 1.0);
pub const YELLOW_500: FlexColor = color(1.0, 0.92, 0.23, 1.0);
pub const YELLOW_700: FlexColor = color(0.98, 0.75, 0.18, 1.0);
pub const YELLOW_900: FlexColor = color(0.96, 0.5, 0.09, 1.0);
pub const PINK_500: FlexColor = color(0.91, 0.12, 0.39, 1.0);
pub const PINK_700: FlexColor = color(0.76, 0.09, 0.36, 1.0);
pub const PINK_900: FlexColor = color(0.53, 0.05, 0.31, 1.0);
pub const ORANGE_500: FlexColor = color(1.0, 0.6, 0.0, 1.0);
pub const ORANGE_700: FlexColor = color(0.96, 0.49, 0.0, 1.0);
pub const ORANGE_900: FlexColor = color(0.9, 0.32, 0.0, 1.0);

/// Converts a [FlexColor] into a hex code representation.
pub fn as_hex(color: FlexColor) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        (color.red * 255.0).round() as i32,
        (color.green * 255.0).round() as i32,
        (color.blue * 255.0).round() as i32
    )
}

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
    NormalCardTitle,
    MortalCardTitle,
    InfernalCardTitle,
    AbyssalCardTitle,
    PrismaticCardTitle,
    ConstructCardTitle,
}

impl From<FontColor> for FlexColor {
    fn from(color: FontColor) -> Self {
        match color {
            FontColor::PrimaryText => WHITE,
            FontColor::ButtonLabel => WHITE,
            FontColor::PanelTitle => WHITE,
            FontColor::NormalCardTitle => BLACK,
            FontColor::MortalCardTitle => BLUE_700,
            FontColor::InfernalCardTitle => RED_600,
            FontColor::AbyssalCardTitle => GREEN_700,
            FontColor::PrismaticCardTitle => ORANGE_900,
            FontColor::ConstructCardTitle => PINK_700,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FontSize {
    ButtonLabel,
    ButtonLabelTwoLines,
    ButtonIcon,
    PanelTitle,
    PromptContext,
    SupplementalInfo,
}

impl From<FontSize> for Dimension {
    fn from(size: FontSize) -> Self {
        (match size {
            FontSize::ButtonLabel => 32,
            FontSize::ButtonLabelTwoLines => 28,
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
