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

use data::actions::UserAction;
use protos::spelldawn::{
    game_action, BorderColor, BorderRadius, BorderWidth, CommandList, Dimension, DimensionGroup,
    DimensionUnit, EventHandlers, FlexColor, FlexRotate, FlexScale, FlexTranslate, FlexVector3,
    GameAction, ImageSlice, Node, SpriteAddress, StandardAction, TimeValue,
};

pub type Px = f32;
pub type Percent = f32;

pub trait Component {
    fn render(self) -> Node;

    fn child(self) -> Option<Node>
    where
        Self: Sized,
    {
        Some(self.render())
    }
}

pub fn child(component: impl Component) -> Option<Node> {
    Some(component.render())
}

pub fn node(component: impl Component) -> Node {
    component.render()
}

/// Turns a [UserAction] into a [GameAction] to be invoked in the future.
///
/// An `optimistic` value can also optionally be provided to give commands to
/// run immediately, before a server response is received.
pub fn action(action: Option<UserAction>, optimistic: Option<CommandList>) -> Option<GameAction> {
    Some(GameAction {
        action: Some(game_action::Action::StandardAction(StandardAction {
            payload: action.map_or(vec![], |action| {
                bincode::serialize(&action).expect("Serialization failed")
            }),
            update: optimistic,
            debug_payload: None,
        })),
    })
}

/// A dimension in units of density-independent pixels. If you're familiar with
/// 'points' from iOS or 'dp' from Android, this unit is approximately 2x from
/// those (i.e. 44dp = 88px).
pub fn px(value: Px) -> Option<Dimension> {
    Some(Dimension { unit: DimensionUnit::Pixels.into(), value })
}

/// A dimension which is a percentage of the parent container, with 100 meaning
/// 100%.
pub fn percent(value: Percent) -> Option<Dimension> {
    Some(Dimension { unit: DimensionUnit::Percentage.into(), value })
}

pub fn left_top_px(left: Px, top: Px) -> Option<DimensionGroup> {
    dimension_group(px(top), None, None, px(left))
}

pub fn left_top_percent(left: Percent, top: Percent) -> Option<DimensionGroup> {
    dimension_group(percent(top), None, None, percent(left))
}

pub fn right_top_px(right: Px, top: Px) -> Option<DimensionGroup> {
    dimension_group(px(top), px(right), None, None)
}

pub fn all_px(all: Px) -> Option<DimensionGroup> {
    px_pair(all, all)
}

pub fn px_pair(top_bottom: Px, right_left: Px) -> Option<DimensionGroup> {
    dimension_group_px(top_bottom, right_left, top_bottom, right_left)
}

pub fn dimension_group_px(top: Px, right: Px, bottom: Px, left: Px) -> Option<DimensionGroup> {
    dimension_group(px(top), px(right), px(bottom), px(left))
}

pub fn top_px(top: Px) -> Option<DimensionGroup> {
    dimension_group(px(top), None, None, None)
}

pub fn right_px(right: Px) -> Option<DimensionGroup> {
    dimension_group(None, px(right), None, None)
}

pub fn bottom_px(bottom: Px) -> Option<DimensionGroup> {
    dimension_group(None, None, px(bottom), None)
}

pub fn left_px(left: Px) -> Option<DimensionGroup> {
    dimension_group(None, None, None, px(left))
}

pub fn dimension_group(
    top: Option<Dimension>,
    right: Option<Dimension>,
    bottom: Option<Dimension>,
    left: Option<Dimension>,
) -> Option<DimensionGroup> {
    Some(DimensionGroup { top, right, bottom, left })
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

pub fn border_radius_px(radius: Px) -> Option<BorderRadius> {
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

pub fn translate_px(x: Px, y: Px) -> Option<FlexTranslate> {
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
