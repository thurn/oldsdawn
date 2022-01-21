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

//! Core reusable UI elements

use protos::spelldawn::{
    node_type, Dimension, EventHandlers, FlexAlign, FlexColor, FlexDirection, FlexJustify,
    FlexPosition, FlexStyle, GameAction, Node, NodeType, SpriteAddress, TextAlign,
};

use crate::core::*;
use crate::macros::children;
use crate::primitives::*;

/// Row flexbox component, renders its children horizontally in sequence
#[derive(Debug, Clone, Default)]
pub struct Row {
    pub name: String,
    pub style: FlexStyle,
    pub children: Vec<Option<Node>>,
    pub events: Option<EventHandlers>,
    pub hover_style: Option<FlexStyle>,
    pub pressed_style: Option<FlexStyle>,
}

impl Component for Row {
    fn render(self) -> Node {
        make_node(
            FlexDirection::Row,
            self.name,
            self.style,
            self.children,
            self.events,
            self.hover_style,
            self.pressed_style,
        )
    }
}

/// Column flexbox component, renders its children vertically in sequence
#[derive(Debug, Clone, Default)]
pub struct Column {
    pub name: String,
    pub style: FlexStyle,
    pub children: Vec<Option<Node>>,
    pub events: Option<EventHandlers>,
    pub hover_style: Option<FlexStyle>,
    pub pressed_style: Option<FlexStyle>,
}

impl Component for Column {
    fn render(self) -> Node {
        make_node(
            FlexDirection::Column,
            self.name,
            self.style,
            self.children,
            self.events,
            self.hover_style,
            self.pressed_style,
        )
    }
}

fn make_node(
    direction: FlexDirection,
    name: String,
    style: FlexStyle,
    children: Vec<Option<Node>>,
    events: Option<EventHandlers>,
    hover_style: Option<FlexStyle>,
    pressed_style: Option<FlexStyle>,
) -> Node {
    Node {
        id: "".to_string(),
        name,
        node_type: None,
        style: Some(FlexStyle { flex_direction: direction.into(), ..style }),
        children: children.into_iter().flatten().collect(),
        event_handlers: events,
        hover_style,
        pressed_style,
    }
}

/// Possible types of [Text]
#[derive(Debug, Clone, Copy)]
pub enum TextVariant {
    /// Describes the contents of a panel.
    PanelTitle,
    /// Text which appears inside a button, use [Button] instead of using this
    /// directly.
    Button,
    /// Text which appears inside a two line button
    TwoLineButton,
    /// Text used to render an IconButton icon
    ButtonIcon,
}

impl TextVariant {
    fn color(self) -> Option<FlexColor> {
        match self {
            TextVariant::PanelTitle => color(Color::TitleText),
            TextVariant::Button => color(Color::ButtonLabel),
            TextVariant::TwoLineButton => color(Color::ButtonLabel),
            TextVariant::ButtonIcon => color(Color::ButtonLabel),
        }
    }

    fn font_size(self) -> Option<Dimension> {
        match self {
            TextVariant::PanelTitle => px(48.0),
            TextVariant::Button => px(32.0),
            TextVariant::TwoLineButton => px(26.0),
            TextVariant::ButtonIcon => px(48.0),
        }
    }

    fn font(self) -> Font {
        match self {
            TextVariant::PanelTitle => Font::Title,
            _ => Font::Default,
        }
    }
}

/// Renders a piece of text in the UI
#[derive(Debug, Clone)]
pub struct Text {
    pub label: String,
    pub variant: TextVariant,
    pub style: FlexStyle,
}

impl Default for Text {
    fn default() -> Self {
        Self {
            label: "".to_string(),
            variant: TextVariant::PanelTitle,
            style: FlexStyle::default(),
        }
    }
}

impl Component for Text {
    fn render(self) -> Node {
        Node {
            node_type: Some(NodeType {
                node_type: Some(node_type::NodeType::Text(protos::spelldawn::Text {
                    label: self.label,
                })),
            }),
            style: Some(FlexStyle {
                padding: all_px(0.0),
                color: self.variant.color(),
                font_size: self.variant.font_size(),
                font: font(self.variant.font()),
                ..self.style
            }),
            ..Node::default()
        }
    }
}

/// Possible types of [Button]
#[derive(Debug, Clone, Copy)]
pub enum ButtonVariant {
    /// Brightly-colored button, main call to action
    Primary,
    /// Less colorful button, deemphasized action
    Secondary,
}

impl ButtonVariant {
    fn background_image(self) -> Option<SpriteAddress> {
        match self {
            ButtonVariant::Primary => sprite(
                "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/Buttons/Rescaled/Button_Orange",
            ),
            ButtonVariant::Secondary => sprite(
                "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/Buttons/Rescaled/Button_Gray",
            ),
        }
    }
}

/// Number of lines of text shown in button
#[derive(Debug, Clone, Copy)]
pub enum ButtonLines {
    OneLine,
    TwoLines,
}

/// Renders a button in the UI
#[derive(Debug, Clone)]
pub struct Button {
    pub label: String,
    pub variant: ButtonVariant,
    pub action: Option<GameAction>,
    pub style: FlexStyle,
    pub lines: ButtonLines,
}

impl Default for Button {
    fn default() -> Self {
        Self {
            label: "".to_string(),
            variant: ButtonVariant::Primary,
            action: None,
            style: FlexStyle::default(),
            lines: ButtonLines::OneLine,
        }
    }
}

impl Component for Button {
    fn render(self) -> Node {
        node(Row {
            name: format!("Button {}", self.label),
            style: FlexStyle {
                height: px(88.0),
                min_width: px(132.0),
                justify_content: FlexJustify::Center.into(),
                align_items: FlexAlign::Center.into(),
                flex_shrink: Some(0.0),
                background_image: self.variant.background_image(),
                image_slice: image_slice(0, 16),
                ..self.style
            },
            children: children!(Text {
                label: self.label,
                variant: match self.lines {
                    ButtonLines::OneLine => TextVariant::Button,
                    ButtonLines::TwoLines => TextVariant::TwoLineButton,
                },
                style: FlexStyle {
                    margin: px_group_2(0.0, 16.0),
                    text_align: TextAlign::MiddleCenter.into(),
                    ..FlexStyle::default()
                },
                ..Text::default()
            }),
            events: self.action.map(|action| EventHandlers { on_click: Some(action) }),
            ..Row::default()
        })
    }
}

#[derive(Debug, Clone, Default)]
pub struct IconButton {
    pub icon: &'static str,
    pub action: Option<GameAction>,
    pub style: FlexStyle,
}

impl Component for IconButton {
    fn render(self) -> Node {
        node(Row {
            name: "IconButton".to_string(),
            style: FlexStyle {
                height: px(88.0),
                width: px(88.0),
                justify_content: FlexJustify::Center.into(),
                align_items: FlexAlign::Center.into(),
                flex_shrink: Some(0.0),
                ..self.style
            },
            events: self.action.map(|action| EventHandlers { on_click: Some(action) }),
            children: children!(
                Row {
                    style: FlexStyle {
                        position: FlexPosition::Absolute.into(),
                        inset: all_px(6.0),
                        height: px(76.0),
                        width: px(76.0),
                        background_image: sprite("Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/Buttons/Square/EPIC_silver_fr_s"),
                        ..FlexStyle::default()
                    },
                    ..Row::default()
                },
                Row {
                    style: FlexStyle {
                        position: FlexPosition::Absolute.into(),
                        inset: all_px(16.0),
                        height: px(56.0),
                        width: px(56.0),
                        background_image: sprite("Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/Buttons/Square/Button_RED_s"),
                        ..FlexStyle::default()
                    },
                    ..Row::default()
                },
                Text {
                    label: self.icon.to_string(),
                    variant: TextVariant::ButtonIcon,
                    style: FlexStyle {
                        text_align: TextAlign::MiddleCenter.into(),
                        ..FlexStyle::default()
                    },
            }),
            ..Row::default()
        })
    }
}
