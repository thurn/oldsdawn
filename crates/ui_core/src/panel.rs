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

#![allow(dead_code)]

use protos::spelldawn::game_command::Command;
use protos::spelldawn::panel_address::AddressType;
use protos::spelldawn::{
    FlexAlign, FlexJustify, FlexPosition, ImageScaleMode, KnownPanelAddress, PanelAddress,
    TextAlign, TogglePanelCommand,
};

use crate::button::IconButton;
use crate::component::EmptyComponent;
use crate::design::{Font, FontColor, FontSize};
use crate::prelude::*;
use crate::style::Pixels;
use crate::text::Text;
use crate::{icons, style};

/// Wraps a [KnownPanelAddress] in a [PanelAddress].
pub fn known(address: KnownPanelAddress) -> PanelAddress {
    PanelAddress { address_type: Some(AddressType::KnownPanel(address as i32)) }
}

#[derive(Debug)]
pub struct Panel {
    address: PanelAddress,
    width: Pixels,
    height: Pixels,
    layout: Layout,
    content: Box<dyn Component>,
    title: Option<String>,
    show_close_button: bool,
}

impl Panel {
    pub fn new(address: PanelAddress, width: impl Into<Pixels>, height: impl Into<Pixels>) -> Self {
        Self {
            address,
            width: width.into(),
            height: height.into(),
            layout: Layout::default(),
            content: Box::new(EmptyComponent),
            title: None,
            show_close_button: false,
        }
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn content(mut self, content: impl Component + 'static) -> Self {
        self.content = Box::new(content);
        self
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn show_close_button(mut self, show_close_button: bool) -> Self {
        self.show_close_button = show_close_button;
        self
    }
}

impl Component for Panel {
    fn build(self) -> RenderResult {
        let background = style::sprite("Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/QuarterSize/Basic_window_big_recolored");
        Row::new(self.title.clone().unwrap_or_else(|| "Panel".to_string()))
            .style(
                Style::new()
                    .position_type(FlexPosition::Absolute)
                    .position(Edge::Left, 50.pct())
                    .position(Edge::Top, 50.pct())
                    .translate((-50).pct(), (-50).pct())
                    .width(self.width)
                    .height(self.height)
                    .padding(Edge::All, 32.px())
                    .align_items(FlexAlign::Center)
                    .justify_content(FlexJustify::Center)
                    .background_image(background)
                    .background_image_scale_mode(ImageScaleMode::StretchToFill)
                    .image_slice(Edge::All, 128.px()),
            )
            .child(self.title.map(TitleBar::new))
            .child(self.show_close_button.then(|| {
                IconButton::new(icons::CLOSE)
                    .action(Command::TogglePanel(TogglePanelCommand {
                        panel_address: Some(self.address),
                        open: false,
                    }))
                    .layout(
                        Layout::new()
                            .position_type(FlexPosition::Absolute)
                            .position(Edge::Right, (-20).px())
                            .position(Edge::Top, (-20).px()),
                    )
            }))
            .child_boxed(self.content)
            .build()
    }
}

#[derive(Debug)]
pub struct TitleBar {
    title: String,
}

impl TitleBar {
    pub fn new(title: impl Into<String>) -> Self {
        Self { title: title.into() }
    }
}

impl Component for TitleBar {
    fn build(self) -> RenderResult {
        let background = style::sprite(
            "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/QuarterSize/Basic_big_bar_512",
        );
        Row::new(format!("TitleBar {}", self.title))
            .style(Style::new().position_type(FlexPosition::Absolute).position(Edge::All, 0.px()))
            .child(
                Row::new("TitleBarContent")
                    .style(
                        Style::new()
                            .position_type(FlexPosition::Absolute)
                            .position(Edge::Left, 50.pct())
                            .position(Edge::Top, 0.pct())
                            .translate((-50).pct(), (-50).pct())
                            .align_items(FlexAlign::Center)
                            .justify_content(FlexJustify::Center)
                            .padding(Edge::Vertical, 16.px())
                            .padding(Edge::Horizontal, 32.px())
                            .background_image(background)
                            .background_image_scale_mode(ImageScaleMode::StretchToFill)
                            .image_slice(Edge::All, 64.px()),
                    )
                    .child(
                        Text::new(self.title, FontSize::PanelTitle)
                            .color(FontColor::PanelTitle)
                            .font(Font::PanelTitle)
                            .text_align(TextAlign::MiddleCenter),
                    ),
            )
            .build()
    }
}
