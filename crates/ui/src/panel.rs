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

use protos::spelldawn::game_action::Action;
use protos::spelldawn::{
    FlexAlign, FlexJustify, FlexPosition, FlexStyle, GameAction, ImageScaleMode, Node,
    PanelAddress, TextAlign, TogglePanelAction,
};

use crate::components::{IconButton, Row, Text, TextVariant};
use crate::core::{Px, *};
use crate::macros::children;
use crate::{icons, Component};

#[derive(Debug, Clone, Default)]
pub struct Panel<TContent: Component> {
    pub address: PanelAddress,
    pub content: TContent,
    pub width: Px,
    pub height: Px,
    pub title: Option<String>,
    pub show_close_button: bool,
}

impl<TContent: Component> Component for Panel<TContent> {
    fn render(self) -> Node {
        node(Row {
            name: self.title.clone().unwrap_or_else(|| "Panel".to_string()),
            style: FlexStyle {
                position: FlexPosition::Absolute.into(),
                inset: left_top_percent(50.0, 50.0),
                translate: translate_percent(-50.0, -50.0),
                width: px(1024.0),
                height: px(512.0),
                padding: all_px(32.0),
                align_items: FlexAlign::Center.into(),
                justify_content: FlexJustify::Center.into(),
                background_image: sprite(
                    "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/QuarterSize/Basic_window_big_recolored",
                ),
                background_image_scale_mode: ImageScaleMode::StretchToFill.into(),
                image_slice: image_slice(128, 128),
                ..FlexStyle::default()
            },
            children: children![
                self.title.map(|title| TitleBar {
                        title,
                        ..TitleBar::default()
                    }),
                self.show_close_button.then(||
                    IconButton {
                        icon: icons::CLOSE,
                        action: Some(GameAction {
                            action: Some(Action::TogglePanel(TogglePanelAction {
                                panel_address: self.address.into(),
                                open: false,
                            }))
                        }),
                        style: FlexStyle {
                            position: FlexPosition::Absolute.into(),
                            inset: right_top_px(-20.0, -20.0),
                            ..FlexStyle::default()
                        },
                        ..IconButton::default()
                    }
                ),
                self.content
            ],
            ..Row::default()
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct TitleBar {
    pub title: String,
}
impl Component for TitleBar {
    fn render(self) -> Node {
        node(Row {
            name: format!("TitleBar {}", self.title),
            style: FlexStyle {
                position: FlexPosition::Absolute.into(),
                inset: all_px(0.0),
                ..FlexStyle::default()
            },
            children: children![Row {
                name: "TitleBarContent".to_string(),
                style: FlexStyle {
                    position: FlexPosition::Absolute.into(),
                    inset: left_top_percent(50.0, 0.0),
                    translate: translate_percent(-50.0, -50.0),
                    align_items: FlexAlign::Center.into(),
                    justify_content: FlexJustify::Center.into(),
                    padding: px_group_2(16.0, 32.0),
                    background_image: sprite(
                        "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/QuarterSize/Basic_big_bar_512",
                    ),
                    background_image_scale_mode: ImageScaleMode::StretchToFill.into(),
                    image_slice: image_slice(64, 64),
                    ..FlexStyle::default()
                },
                children: children![Text {
                    label: self.title,
                    variant: TextVariant::PanelTitle,
                    style: FlexStyle {
                        text_align: TextAlign::MiddleCenter.into(),
                        ..FlexStyle::default()
                    },
                    ..Text::default()
                }],
                ..Row::default()
            }],
            ..Row::default()
        })
    }
}
