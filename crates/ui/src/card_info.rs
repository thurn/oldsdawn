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

use protos::spelldawn::{FlexAlign, FlexJustify, FlexStyle, Node, TextAlign, WhiteSpace};

use crate::components::{Column, Row, Text};
use crate::core::*;
use crate::{colors, font_sizes, fonts, Component};

/// Renders helper text for a card, displayed during an info zoom.
#[derive(Debug, Default)]
pub struct SupplementalCardInfo {
    pub info: Vec<String>,
}

impl Component for SupplementalCardInfo {
    fn render(self) -> Node {
        Column {
            name: "SupplementalInfo".to_string(),
            style: FlexStyle {
                align_items: FlexAlign::FlexStart.into(),
                justify_content: FlexJustify::FlexStart.into(),
                margin: px_pair(0.0, 16.0),
                max_width: px(600.0),
                max_height: px(600.0),
                ..FlexStyle::default()
            },
            children: self
                .info
                .into_iter()
                .enumerate()
                .map(|(i, text)| InfoNode { is_first: i == 0, text }.child())
                .collect(),
            ..Column::default()
        }
        .render()
    }
}

/// A single node of supplemental card info
#[derive(Debug, Default)]
pub struct InfoNode {
    pub is_first: bool,
    pub text: String,
}

impl Component for InfoNode {
    fn render(self) -> Node {
        Row {
            name: "InfoNode".to_string(),
            style: FlexStyle {
                margin: if self.is_first { bottom_px(4.0) } else { px_pair(4.0, 0.0) },
                background_color: colors::CARD_INFO_BACKGROUND,
                border_radius: border_radius_px(12.0),
                justify_content: FlexJustify::Center.into(),
                align_items: FlexAlign::Center.into(),
                ..FlexStyle::default()
            },
            children: vec![Text {
                label: self.text,
                color: colors::SUPPLEMENTAL_INFO_TEXT,
                font_size: font_sizes::SUPPLEMENTAL_INFO_TEXT,
                font: fonts::SUPPLEMENTAL_INFO_TEXT,
                style: FlexStyle {
                    margin: all_px(16.0),
                    text_align: TextAlign::MiddleLeft.into(),
                    white_space: WhiteSpace::Normal.into(),
                    ..FlexStyle::default()
                },
            }
            .child()],
            ..Row::default()
        }
        .render()
    }
}
