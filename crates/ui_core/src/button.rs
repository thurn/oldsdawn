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

use data::game_actions::UserAction;
use protos::spelldawn::{FlexAlign, FlexJustify, FlexPosition, TextAlign};

use crate::design::{Font, FontColor, FontSize};
use crate::prelude::*;
use crate::style;
use crate::text::Text;

#[derive(Debug, Clone, Copy)]
pub enum ButtonType {
    /// Brightly-colored button, main call to action
    Primary,
    /// Less colorful button, deemphasized action
    Secondary,
}

#[derive(Debug, Clone, Copy)]
pub enum ButtonTextSize {
    Default,
    Multiline,
}

/// Implements a standard clickable button
#[derive(Debug)]
pub struct Button {
    label: String,
    layout: Layout,
    button_type: ButtonType,
    action: Option<UserAction>,
}

impl Button {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            layout: Layout::default(),
            button_type: ButtonType::Primary,
            action: None,
        }
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn button_type(mut self, button_type: ButtonType) -> Self {
        self.button_type = button_type;
        self
    }

    pub fn action(mut self, action: UserAction) -> Self {
        self.action = Some(action);
        self
    }
}

impl Component for Button {
    fn render(self) -> RenderResult {
        let background = style::sprite(match self.button_type {
            ButtonType::Primary => {
                "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/Buttons/Rescaled/Button_Orange"
            }
            ButtonType::Secondary => {
                "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/Buttons/Rescaled/Button_Gray"
            }
        });

        Row::new(format!("{} Button", self.label))
            .style(
                self.layout
                    .to_style()
                    .height(88.px())
                    .min_width(132.px())
                    .justify_content(FlexJustify::Center)
                    .align_items(FlexAlign::Center)
                    .flex_shrink(0.0)
                    .background_image(background)
                    .image_slice(Edge::Horizontal, 16.px()),
            )
            .on_click(self.action)
            .child(
                Text::new(self.label, FontSize::ButtonLabel)
                    .color(FontColor::ButtonLabel)
                    .font(Font::ButtonLabel)
                    .text_align(TextAlign::MiddleCenter)
                    .layout(Layout::new().margin(Edge::Horizontal, 16.px())),
            )
            .build()
    }
}

#[derive(Debug, Clone, Copy)]
pub enum IconButtonType {
    /// Red button background, used to close a window
    Close,
}

#[derive(Debug)]
pub struct IconButton {
    icon: String,
    layout: Layout,
    button_type: IconButtonType,
    action: Option<UserAction>,
}

impl IconButton {
    pub fn new(icon: impl Into<String>) -> Self {
        Self {
            icon: icon.into(),
            layout: Layout::default(),
            button_type: IconButtonType::Close,
            action: None,
        }
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn button_type(mut self, button_type: IconButtonType) -> Self {
        self.button_type = button_type;
        self
    }

    pub fn action(mut self, action: UserAction) -> Self {
        self.action = Some(action);
        self
    }
}

impl Component for IconButton {
    fn render(self) -> RenderResult {
        let frame = style::sprite(
            "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/Buttons/Square/EPIC_silver_fr_s",
        );
        let background = style::sprite(match self.button_type {
            IconButtonType::Close => {
                "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/Buttons/Square/Button_RED_s"
            }
        });

        Row::new("IconButton")
            .style(
                self.layout
                    .to_style()
                    .height(88.px())
                    .width(88.px())
                    .justify_content(FlexJustify::Center)
                    .align_items(FlexAlign::Center)
                    .flex_shrink(0.0),
            )
            .on_click(self.action)
            .child(
                Row::new("Frame").style(
                    Style::new()
                        .position_type(FlexPosition::Absolute)
                        .position(Edge::All, 6.px())
                        .height(76.px())
                        .width(76.px())
                        .background_image(frame),
                ),
            )
            .child(
                Row::new("Background").style(
                    Style::new()
                        .position_type(FlexPosition::Absolute)
                        .position(Edge::All, 16.px())
                        .height(56.px())
                        .width(56.px())
                        .background_image(background),
                ),
            )
            .child(
                Text::new(self.icon, FontSize::ButtonIcon)
                    .color(FontColor::ButtonLabel)
                    .font(Font::ButtonLabel)
                    .text_align(TextAlign::MiddleCenter),
            )
            .build()
    }
}
