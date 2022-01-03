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

//! Common helper functions for defining user interface elements, intended to be
//! used via wildcard import. All functions in this module return [Option]
//! despite the fact that they are infallible, since all of the higher-level
//! APIs exclusively consume [Option] types.

use protos::spelldawn::{
    game_action, BorderColor, BorderRadius, BorderWidth, Dimension, DimensionGroup, DimensionUnit,
    EventHandlers, FlexColor, FlexDirection, FlexRotate, FlexScale, FlexStyle, FlexTranslate,
    FlexVector3, GameAction, ImageSlice, Node, SpriteAddress, StandardAction, TimeValue,
};

/// Macro which converts its arguments into a vector of [Node]s via
/// [Node::from].
macro_rules! children {
    ($($x:expr),*) => {
        vec! [$(protos::spelldawn::Node::from($x)),*]
    };
    ($($x:expr,)*) => {children![$($x),*]}
}

pub(crate) use children;

/// Helper function to create a node with 'row' flex direction
pub fn row(name: impl Into<String>, style: FlexStyle, children: Vec<Node>) -> Node {
    make_flexbox(name, style, children, FlexDirection::Row)
}

/// Helper function to create a node with 'column' flex direction
pub fn column(name: impl Into<String>, style: FlexStyle, children: Vec<Node>) -> Node {
    make_flexbox(name, style, children, FlexDirection::Column)
}

/// A dimension in units of density-independent pixels
pub fn px(value: f32) -> Option<Dimension> {
    Some(Dimension { unit: DimensionUnit::Pixels.into(), value })
}

/// A dimension which is a percentage of the parent container
pub fn percent(value: f32) -> Option<Dimension> {
    Some(Dimension { unit: DimensionUnit::Percentage.into(), value })
}

pub fn left_top_px(left: f32, top: f32) -> Option<DimensionGroup> {
    group_px(top, 0.0, 0.0, left)
}

pub fn all_px(all: f32) -> Option<DimensionGroup> {
    group_px(all, all, all, all)
}

pub fn left_right_px(left_right: f32) -> Option<DimensionGroup> {
    group_px(0.0, left_right, 0.0, left_right)
}

pub fn top_bottom_px(top_bottom: f32) -> Option<DimensionGroup> {
    group_px(top_bottom, 0.0, top_bottom, 0.0)
}

pub fn top_px(top: f32) -> Option<DimensionGroup> {
    group_px(top, 0.0, 0.0, 0.0)
}

pub fn right_px(right: f32) -> Option<DimensionGroup> {
    group_px(0.0, right, 0.0, 0.0)
}

pub fn bottom_px(bottom: f32) -> Option<DimensionGroup> {
    group_px(0.0, 0.0, bottom, 0.0)
}

pub fn left_px(left: f32) -> Option<DimensionGroup> {
    group_px(0.0, 0.0, 0.0, left)
}

pub fn group_px(top: f32, right: f32, bottom: f32, left: f32) -> Option<DimensionGroup> {
    Some(DimensionGroup { top: px(top), right: px(right), bottom: px(bottom), left: px(left) })
}

pub fn border_color(color: Option<FlexColor>) -> Option<BorderColor> {
    Some(BorderColor {
        top: color.clone(),
        right: color.clone(),
        bottom: color.clone(),
        left: color,
    })
}

pub fn border_width(width: f32) -> Option<BorderWidth> {
    Some(BorderWidth { top: width, right: width, bottom: width, left: width })
}

pub fn border_radius_px(radius: f32) -> Option<BorderRadius> {
    Some(BorderRadius {
        top_left: px(radius),
        top_right: px(radius),
        bottom_right: px(radius),
        bottom_left: px(radius),
    })
}

pub fn sprite(address: &str) -> Option<SpriteAddress> {
    Some(SpriteAddress { address: address.to_string() })
}

pub fn scale(amount: f32) -> Option<FlexScale> {
    Some(FlexScale { amount: Some(FlexVector3 { x: amount, y: amount, z: 0.0 }) })
}

pub fn rotate(degrees: f32) -> Option<FlexRotate> {
    Some(FlexRotate { degrees })
}

pub fn translate_px(x: f32, y: f32) -> Option<FlexTranslate> {
    Some(FlexTranslate { x: px(x), y: px(y), z: 0.0 })
}

pub fn translate_percent(x: f32, y: f32) -> Option<FlexTranslate> {
    Some(FlexTranslate { x: percent(x), y: percent(y), z: 0.0 })
}

pub fn duration_ms(milliseconds: u32) -> Option<TimeValue> {
    Some(TimeValue { milliseconds })
}

/// Creates an [ImageSlice] to allow an image to scale via 9-slicing with the
/// provided top/bottom and left/right slice dimensions.
pub fn image_slice(top_bottom: u32, left_right: u32) -> Option<ImageSlice> {
    Some(ImageSlice { top: top_bottom, right: left_right, bottom: top_bottom, left: left_right })
}

/// Creates a handler to invoke a [StandardAction] on click.
pub fn on_click(action: StandardAction) -> Option<EventHandlers> {
    Some(EventHandlers {
        on_click: Some(GameAction { action: Some(game_action::Action::StandardAction(action)) }),
    })
}

fn make_flexbox(
    name: impl Into<String>,
    style: FlexStyle,
    children: Vec<Node>,
    direction: FlexDirection,
) -> Node {
    Node {
        name: name.into(),
        style: Some(FlexStyle { flex_direction: direction.into(), ..style }),
        children,
        ..Node::default()
    }
}
