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

use protos::spelldawn::node_background::BackgroundAddress;
use protos::spelldawn::{
    Dimension, DimensionGroup, DimensionUnit, EasingMode, FlexAlign, FlexColor, FlexDirection,
    FlexDisplayStyle, FlexJustify, FlexOverflow, FlexPickingMode, FlexPosition, FlexRotate,
    FlexScale, FlexStyle, FlexTranslate, FlexVector3, FlexVisibility, FlexWrap, FontAddress,
    FontStyle, ImageScaleMode, NodeBackground, OverflowClipBox, SpriteAddress, TextAlign,
    TextOverflow, TextOverflowPosition, TextShadow, TimeValue, WhiteSpace,
};

/// Pixels unit. Not literally equivalent to screen pixels, Unity resizes these
/// values based on its UI scaling mode.
#[derive(Debug)]
pub struct Pixels(f32);

impl From<Pixels> for Dimension {
    fn from(pixels: Pixels) -> Self {
        Self { unit: DimensionUnit::Pixels as i32, value: pixels.0 }
    }
}

/// Percentage unit, typically based on parent container size.
#[derive(Debug)]
pub struct Percentage(f32);

impl From<Percentage> for Dimension {
    fn from(percentage: Percentage) -> Self {
        Self { unit: DimensionUnit::Percentage as i32, value: percentage.0 }
    }
}

/// Angular unit, used for rotations
#[derive(Debug)]
pub struct Degrees(f32);

/// Helper trait to create various dimensional units from numeric literals.
pub trait DimensionExt {
    fn px(self) -> Pixels;

    fn pct(self) -> Percentage;

    fn milliseconds(self) -> TimeValue;

    fn degrees(self) -> Degrees;
}

impl DimensionExt for i32 {
    fn px(self) -> Pixels {
        Pixels(self as f32)
    }

    fn pct(self) -> Percentage {
        Percentage(self as f32)
    }

    fn milliseconds(self) -> TimeValue {
        TimeValue { milliseconds: self as u32 }
    }

    fn degrees(self) -> Degrees {
        Degrees(self as f32)
    }
}

impl DimensionExt for f32 {
    fn px(self) -> Pixels {
        Pixels(self)
    }

    fn pct(self) -> Percentage {
        Percentage(self)
    }

    fn milliseconds(self) -> TimeValue {
        TimeValue { milliseconds: self as u32 }
    }

    fn degrees(self) -> Degrees {
        Degrees(self as Self)
    }
}

/// Turns a string into a [SpriteAddress].
pub fn sprite(string: impl Into<String>) -> SpriteAddress {
    SpriteAddress { address: format!("{}.png", string.into()) }
}

/// Controls the growth behavior of a component.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum WidthMode {
    /// Grow to fit container, i.e. flex-grow 1
    Flexible,
    /// Size to fit contents, i.e. flex-grow 0
    Constrained,
}

/// Identifies one or more edges of a flexbox to be styled.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Edge {
    All,
    Vertical,
    Horizontal,
    Top,
    Right,
    Bottom,
    Left,
}

/// Identifies one or more corners of a flexbox to be styled.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Corner {
    All,
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

/// Primary styling type for the UI system. Allows a large number of properties
/// to be changed on an underlying UI node.
#[derive(Debug, Clone, Default)]
pub struct Style {
    wrapped_style: FlexStyle,
}

impl Style {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn wrapped_style(self) -> FlexStyle {
        self.wrapped_style
    }

    pub fn align_content(mut self, align_content: FlexAlign) -> Self {
        self.wrapped_style.align_content = align_content as i32;
        self
    }

    pub fn align_items(mut self, align_items: FlexAlign) -> Self {
        self.wrapped_style.align_items = align_items as i32;
        self
    }

    pub fn align_self(mut self, align_self: FlexAlign) -> Self {
        self.wrapped_style.align_self = align_self as i32;
        self
    }

    pub fn background_color(mut self, color: impl Into<FlexColor>) -> Self {
        self.wrapped_style.background_color = Some(color.into());
        self
    }

    pub fn background_image(mut self, sprite: SpriteAddress) -> Self {
        self.wrapped_style.background_image =
            Some(NodeBackground { background_address: Some(BackgroundAddress::Sprite(sprite)) });
        self
    }

    pub fn border_color(mut self, edge: Edge, color: impl Into<FlexColor>) -> Self {
        self.wrapped_style.border_color = apply_edge(
            edge,
            color.into(),
            &self.wrapped_style.border_color,
            |parent, setter, value| match setter {
                EdgeSetter::Top => parent.top = value,
                EdgeSetter::Right => parent.right = value,
                EdgeSetter::Bottom => parent.bottom = value,
                EdgeSetter::Left => parent.left = value,
            },
        );
        self
    }

    pub fn border_radius(mut self, corner: Corner, radius: impl Into<Dimension>) -> Self {
        self.wrapped_style.border_radius = apply_corner(
            corner,
            radius.into(),
            &self.wrapped_style.border_radius,
            |parent, setter, value| match setter {
                CornerSetter::TopLeft => parent.top_left = value,
                CornerSetter::TopRight => parent.top_right = value,
                CornerSetter::BottomRight => parent.bottom_right = value,
                CornerSetter::BottomLeft => parent.bottom_left = value,
            },
        );
        self
    }

    pub fn border_width(mut self, edge: Edge, width: Pixels) -> Self {
        self.wrapped_style.border_width =
            apply_edge(edge, width.0, &self.wrapped_style.border_width, |parent, setter, value| {
                match setter {
                    EdgeSetter::Top => parent.top = value.unwrap(),
                    EdgeSetter::Right => parent.right = value.unwrap(),
                    EdgeSetter::Bottom => parent.bottom = value.unwrap(),
                    EdgeSetter::Left => parent.left = value.unwrap(),
                }
            });
        self
    }

    pub fn position(mut self, edge: Edge, size: impl Into<Dimension>) -> Self {
        self.wrapped_style.inset =
            apply_edge(edge, size.into(), &self.wrapped_style.inset, |parent, setter, value| {
                match setter {
                    EdgeSetter::Top => parent.top = value,
                    EdgeSetter::Right => parent.right = value,
                    EdgeSetter::Bottom => parent.bottom = value,
                    EdgeSetter::Left => parent.left = value,
                }
            });
        self
    }

    pub fn color(mut self, color: impl Into<FlexColor>) -> Self {
        self.wrapped_style.color = Some(color.into());
        self
    }

    pub fn display(mut self, display: FlexDisplayStyle) -> Self {
        self.wrapped_style.display = display as i32;
        self
    }

    pub fn flex_basis(mut self, basis: impl Into<Dimension>) -> Self {
        self.wrapped_style.flex_basis = Some(basis.into());
        self
    }

    pub fn flex_direction(mut self, direction: FlexDirection) -> Self {
        self.wrapped_style.flex_direction = direction as i32;
        self
    }

    pub fn flex_grow(mut self, grow: f32) -> Self {
        self.wrapped_style.flex_grow = Some(grow);
        self
    }

    pub fn flex_shrink(mut self, shrink: f32) -> Self {
        self.wrapped_style.flex_shrink = Some(shrink);
        self
    }

    pub fn wrap(mut self, wrap: FlexWrap) -> Self {
        self.wrapped_style.wrap = wrap as i32;
        self
    }

    pub fn font_size(mut self, size: impl Into<Dimension>) -> Self {
        self.wrapped_style.font_size = Some(size.into());
        self
    }

    pub fn height(mut self, height: impl Into<Dimension>) -> Self {
        self.wrapped_style.height = Some(height.into());
        self
    }

    pub fn width(mut self, width: impl Into<Dimension>) -> Self {
        self.wrapped_style.width = Some(width.into());
        self
    }

    pub fn justify_content(mut self, justify_content: FlexJustify) -> Self {
        self.wrapped_style.justify_content = justify_content as i32;
        self
    }

    pub fn letter_spacing(mut self, spacing: impl Into<Dimension>) -> Self {
        self.wrapped_style.letter_spacing = Some(spacing.into());
        self
    }

    pub fn margin(mut self, edge: Edge, size: impl Into<Dimension>) -> Self {
        self.wrapped_style.margin =
            apply_dimension_group(&self.wrapped_style.margin, edge, size.into());
        self
    }

    pub fn max_height(mut self, height: impl Into<Dimension>) -> Self {
        self.wrapped_style.max_height = Some(height.into());
        self
    }

    pub fn max_width(mut self, width: impl Into<Dimension>) -> Self {
        self.wrapped_style.max_width = Some(width.into());
        self
    }

    pub fn min_height(mut self, height: impl Into<Dimension>) -> Self {
        self.wrapped_style.min_height = Some(height.into());
        self
    }

    pub fn min_width(mut self, width: impl Into<Dimension>) -> Self {
        self.wrapped_style.min_width = Some(width.into());
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.wrapped_style.opacity = Some(opacity);
        self
    }

    pub fn overflow(mut self, overflow: FlexOverflow) -> Self {
        self.wrapped_style.overflow = overflow as i32;
        self
    }

    pub fn padding(mut self, edge: Edge, size: impl Into<Dimension>) -> Self {
        self.wrapped_style.padding =
            apply_dimension_group(&self.wrapped_style.padding, edge, size.into());
        self
    }

    pub fn position_type(mut self, position_type: FlexPosition) -> Self {
        self.wrapped_style.position = position_type as i32;
        self
    }

    pub fn rotate(mut self, degrees: Degrees) -> Self {
        self.wrapped_style.rotate = Some(FlexRotate { degrees: degrees.0 });
        self
    }

    pub fn scale(mut self, scale: FlexVector3) -> Self {
        self.wrapped_style.scale = Some(FlexScale { amount: Some(scale) });
        self
    }

    pub fn text_overflow(mut self, overflow: TextOverflow) -> Self {
        self.wrapped_style.overflow = overflow as i32;
        self
    }

    pub fn text_shadow(mut self, shadow: TextShadow) -> Self {
        self.wrapped_style.text_shadow = Some(shadow);
        self
    }

    pub fn transform_origin(self, x: impl Into<Dimension>, y: impl Into<Dimension>) -> Self {
        self.transform_origin_with_z(x, y, 0.0)
    }

    pub fn transform_origin_with_z(
        mut self,
        x: impl Into<Dimension>,
        y: impl Into<Dimension>,
        z: f32,
    ) -> Self {
        self.wrapped_style.transform_origin =
            Some(FlexTranslate { x: Some(x.into()), y: Some(y.into()), z });
        self
    }

    pub fn transition_delays(mut self, delays: Vec<TimeValue>) -> Self {
        self.wrapped_style.transition_delays = delays;
        self
    }

    pub fn transition_durations(mut self, durations: Vec<TimeValue>) -> Self {
        self.wrapped_style.transition_durations = durations;
        self
    }

    pub fn transition_properties(mut self, properties: Vec<String>) -> Self {
        self.wrapped_style.transition_properties = properties;
        self
    }

    pub fn transition_easing_modes(mut self, modes: Vec<EasingMode>) -> Self {
        self.wrapped_style.transition_easing_modes = modes.into_iter().map(|m| m as i32).collect();
        self
    }

    pub fn translate(self, x: impl Into<Dimension>, y: impl Into<Dimension>) -> Self {
        self.translate_with_z(x, y, 0.0)
    }

    pub fn translate_with_z(
        mut self,
        x: impl Into<Dimension>,
        y: impl Into<Dimension>,
        z: f32,
    ) -> Self {
        self.wrapped_style.translate =
            Some(FlexTranslate { x: Some(x.into()), y: Some(y.into()), z });
        self
    }

    pub fn background_image_tint_color(mut self, color: impl Into<FlexColor>) -> Self {
        self.wrapped_style.background_image_tint_color = Some(color.into());
        self
    }

    pub fn background_image_scale_mode(mut self, mode: ImageScaleMode) -> Self {
        self.wrapped_style.background_image_scale_mode = mode as i32;
        self
    }

    pub fn font(mut self, font: impl Into<FontAddress>) -> Self {
        self.wrapped_style.font = Some(font.into());
        self
    }

    pub fn font_style(mut self, style: FontStyle) -> Self {
        self.wrapped_style.font_style = style as i32;
        self
    }

    pub fn overflow_clip_box(mut self, clip: OverflowClipBox) -> Self {
        self.wrapped_style.overflow_clip_box = clip as i32;
        self
    }

    pub fn paragraph_spacing(mut self, spacing: impl Into<Dimension>) -> Self {
        self.wrapped_style.paragraph_spacing = Some(spacing.into());
        self
    }

    pub fn image_slice(mut self, edge: Edge, size: Pixels) -> Self {
        self.wrapped_style.image_slice = apply_edge(
            edge,
            size.0.round() as u32,
            &self.wrapped_style.image_slice,
            |parent, setter, value| match setter {
                EdgeSetter::Top => parent.top = value.unwrap(),
                EdgeSetter::Right => parent.right = value.unwrap(),
                EdgeSetter::Bottom => parent.bottom = value.unwrap(),
                EdgeSetter::Left => parent.left = value.unwrap(),
            },
        );
        self
    }

    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.wrapped_style.text_align = align as i32;
        self
    }

    pub fn text_outline_color(mut self, color: impl Into<FlexColor>) -> Self {
        self.wrapped_style.text_outline_color = Some(color.into());
        self
    }

    pub fn text_outline_width(mut self, width: Pixels) -> Self {
        self.wrapped_style.text_outline_width = Some(width.0);
        self
    }

    pub fn text_overflow_position(mut self, position: TextOverflowPosition) -> Self {
        self.wrapped_style.text_overflow_position = position as i32;
        self
    }

    pub fn visibility(mut self, visibility: FlexVisibility) -> Self {
        self.wrapped_style.visibility = visibility as i32;
        self
    }

    pub fn white_space(mut self, white_space: WhiteSpace) -> Self {
        self.wrapped_style.white_space = white_space as i32;
        self
    }

    pub fn word_spacing(mut self, spacing: impl Into<Dimension>) -> Self {
        self.wrapped_style.word_spacing = Some(spacing.into());
        self
    }

    pub fn picking_mode(mut self, picking_mode: FlexPickingMode) -> Self {
        self.wrapped_style.picking_mode = picking_mode as i32;
        self
    }
}

/// Implements a subset of the full [Style] API for user in higher-level
/// components that don't require arbitrary styling.
#[derive(Debug, Clone, Default)]
pub struct Layout {
    style: Style,
}

impl Layout {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn align_self(mut self, align_self: FlexAlign) -> Self {
        self.style = self.style.align_self(align_self);
        self
    }

    pub fn position(mut self, edge: Edge, size: impl Into<Dimension>) -> Self {
        self.style = self.style.position(edge, size);
        self
    }

    pub fn display(mut self, display: FlexDisplayStyle) -> Self {
        self.style = self.style.display(display);
        self
    }

    pub fn margin(mut self, edge: Edge, size: impl Into<Dimension>) -> Self {
        self.style = self.style.margin(edge, size);
        self
    }

    pub fn opacity(mut self, opacity: f32) -> Self {
        self.style = self.style.opacity(opacity);
        self
    }

    pub fn position_type(mut self, position_type: FlexPosition) -> Self {
        self.style = self.style.position_type(position_type);
        self
    }

    pub fn visibility(mut self, visibility: FlexVisibility) -> Self {
        self.style = self.style.visibility(visibility);
        self
    }

    pub fn to_style(self) -> Style {
        self.style
    }
}

enum EdgeSetter {
    Top,
    Right,
    Bottom,
    Left,
}

enum CornerSetter {
    TopLeft,
    TopRight,
    BottomRight,
    BottomLeft,
}

fn apply_dimension_group(
    current: &Option<DimensionGroup>,
    edge: Edge,
    dimension: Dimension,
) -> Option<DimensionGroup> {
    apply_edge(edge, dimension, current, |parent, setter, value| match setter {
        EdgeSetter::Top => parent.top = value,
        EdgeSetter::Right => parent.right = value,
        EdgeSetter::Bottom => parent.bottom = value,
        EdgeSetter::Left => parent.left = value,
    })
}

fn apply_edge<P: Default + Clone, T: Clone>(
    edge: Edge,
    value: T,
    current: &Option<P>,
    setter: impl Fn(&mut P, EdgeSetter, Option<T>),
) -> Option<P> {
    let mut parent = current.clone().unwrap_or_default();
    match edge {
        Edge::All => {
            setter(&mut parent, EdgeSetter::Top, Some(value.clone()));
            setter(&mut parent, EdgeSetter::Right, Some(value.clone()));
            setter(&mut parent, EdgeSetter::Bottom, Some(value.clone()));
            setter(&mut parent, EdgeSetter::Left, Some(value));
        }
        Edge::Vertical => {
            setter(&mut parent, EdgeSetter::Top, Some(value.clone()));
            setter(&mut parent, EdgeSetter::Bottom, Some(value));
        }
        Edge::Horizontal => {
            setter(&mut parent, EdgeSetter::Left, Some(value.clone()));
            setter(&mut parent, EdgeSetter::Right, Some(value));
        }
        Edge::Top => {
            setter(&mut parent, EdgeSetter::Top, Some(value));
        }
        Edge::Right => {
            setter(&mut parent, EdgeSetter::Right, Some(value));
        }
        Edge::Bottom => {
            setter(&mut parent, EdgeSetter::Bottom, Some(value));
        }
        Edge::Left => {
            setter(&mut parent, EdgeSetter::Left, Some(value));
        }
    }
    Some(parent)
}

fn apply_corner<P: Default + Clone, T: Clone>(
    edge: Corner,
    value: T,
    current: &Option<P>,
    setter: impl Fn(&mut P, CornerSetter, Option<T>),
) -> Option<P> {
    let mut parent = current.clone().unwrap_or_default();
    match edge {
        Corner::All => {
            setter(&mut parent, CornerSetter::TopLeft, Some(value.clone()));
            setter(&mut parent, CornerSetter::TopRight, Some(value.clone()));
            setter(&mut parent, CornerSetter::BottomRight, Some(value.clone()));
            setter(&mut parent, CornerSetter::BottomLeft, Some(value));
        }
        Corner::TopLeft => {
            setter(&mut parent, CornerSetter::TopLeft, Some(value));
        }
        Corner::TopRight => {
            setter(&mut parent, CornerSetter::TopRight, Some(value));
        }
        Corner::BottomRight => {
            setter(&mut parent, CornerSetter::BottomRight, Some(value));
        }
        Corner::BottomLeft => {
            setter(&mut parent, CornerSetter::BottomLeft, Some(value));
        }
    }
    Some(parent)
}
