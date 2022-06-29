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

use std::fmt::Debug;
use std::marker::PhantomData;

use protos::spelldawn::{EventHandlers, FlexDirection, GameAction, Node};

use crate::actions::InterfaceAction;
use crate::component::{Component, RenderResult};
use crate::style::Style;

/// Renders a [Flexbox] which lays out its children horizontally, from left to
/// right
pub type Row = Flexbox<RowDirection>;

/// Renders a [Flexbox] which lays out its children vertically, from top to
/// bottom
pub type Column = Flexbox<ColumnDirection>;

/// Renders a reversed (right-to-left) [Row]
pub type ReverseRow = Flexbox<ReverseRowDirection>;

/// Renders a reversed (bottom-to-top) [Column]
pub type ReverseColumn = Flexbox<ReverseColumnDirection>;

/// Marker trait to control the direction of a [Flexbox]
pub trait FlexboxDirection: Default + Debug {
    fn direction() -> FlexDirection;
}

#[derive(Debug, Default)]
pub struct RowDirection {}

impl FlexboxDirection for RowDirection {
    fn direction() -> FlexDirection {
        FlexDirection::Row
    }
}

#[derive(Debug, Default)]
pub struct ColumnDirection {}

impl FlexboxDirection for ColumnDirection {
    fn direction() -> FlexDirection {
        FlexDirection::Column
    }
}

#[derive(Debug, Default)]
pub struct ReverseRowDirection {}

impl FlexboxDirection for ReverseRowDirection {
    fn direction() -> FlexDirection {
        FlexDirection::RowReverse
    }
}

#[derive(Debug, Default)]
pub struct ReverseColumnDirection {}

impl FlexboxDirection for ReverseColumnDirection {
    fn direction() -> FlexDirection {
        FlexDirection::ColumnReverse
    }
}

/// Marker trait for any type which directly renders a [Node] and be styled by
/// [Style].
pub trait HasRenderNode: Sized {
    fn render_node(&mut self) -> &mut Node;

    fn flex_direction(&self) -> Option<FlexDirection> {
        None
    }

    /// Name for this component. Used for debugging.
    fn name(mut self, name: impl Into<String>) -> Self {
        self.render_node().name = name.into();
        self
    }

    /// Primary [Style] used when the component is not hovered or pressed.
    fn style(mut self, mut style: Style) -> Self {
        if let Some(d) = self.flex_direction() {
            style = style.flex_direction(d);
        }
        self.render_node().style = Some(style.wrapped_style());
        self
    }

    /// [Style] to merge into this component's base style when it is hovered
    fn hover_style(mut self, style: Style) -> Self {
        self.render_node().hover_style = Some(style.wrapped_style());
        self
    }

    /// [Style] to merge into this component's base style when it is pressed
    fn pressed_style(mut self, style: Style) -> Self {
        self.render_node().pressed_style = Some(style.wrapped_style());
        self
    }

    /// Action to invoke when this component is clicked/tapped
    fn on_click(mut self, action: impl InterfaceAction + 'static) -> Self {
        if let Some(action) = action.as_game_action() {
            self.render_node().event_handlers =
                Some(EventHandlers { on_click: Some(GameAction { action: Some(action) }) });
        }
        self
    }
}

/// Primary container component for the UI system. Lays out its children
/// following flexbox spacing rules. Typically used via its [Row] or [Column]
/// aliases.
#[derive(Debug, Default)]
pub struct Flexbox<D: FlexboxDirection> {
    children: Vec<Box<dyn Component>>,
    render_node: Node,
    phantom: PhantomData<D>,
}

impl<D: FlexboxDirection> HasRenderNode for Flexbox<D> {
    fn render_node(&mut self) -> &mut Node {
        &mut self.render_node
    }

    fn flex_direction(&self) -> Option<FlexDirection> {
        Some(D::direction())
    }
}

impl<D: FlexboxDirection> Flexbox<D> {
    pub fn new(name: impl Into<String>) -> Self {
        let mut result = Self::default();
        result.render_node.name = name.into();
        result
    }

    pub fn child(mut self, child: impl Component + 'static) -> Self {
        self.children.push(Box::new(child));
        self
    }

    pub fn child_boxed(mut self, child: Box<dyn Component>) -> Self {
        self.children.push(child);
        self
    }

    pub fn render_node(&mut self) -> &mut Node {
        &mut self.render_node
    }
}

impl<D: FlexboxDirection> Component for Flexbox<D> {
    fn build(self) -> RenderResult {
        RenderResult::Container(Box::new(self.render_node), self.children)
    }
}
