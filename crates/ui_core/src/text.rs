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

use protos::spelldawn::{node_type, FontStyle, Node, NodeType, TextAlign, WhiteSpace};

use crate::design::{Font, FontColor, FontSize};
use crate::prelude::*;

/// Standard design-system-aware text-rendering component
#[derive(Debug)]
pub struct Text {
    text: String,
    size: FontSize,
    color: FontColor,
    font: Font,
    layout: Layout,
    font_style: FontStyle,
    text_align: TextAlign,
    white_space: WhiteSpace,
}

impl Text {
    pub fn new(text: impl Into<String>, size: FontSize) -> Self {
        Self {
            text: text.into(),
            color: FontColor::PrimaryText,
            size,
            font: Font::PrimaryText,
            layout: Layout::default(),
            font_style: FontStyle::Unspecified,
            text_align: TextAlign::Unspecified,
            white_space: WhiteSpace::Unspecified,
        }
    }

    pub fn layout(mut self, layout: Layout) -> Self {
        self.layout = layout;
        self
    }

    pub fn color(mut self, color: FontColor) -> Self {
        self.color = color;
        self
    }

    pub fn font(mut self, font: Font) -> Self {
        self.font = font;
        self
    }

    pub fn font_style(mut self, font_style: FontStyle) -> Self {
        self.font_style = font_style;
        self
    }

    pub fn text_align(mut self, align: TextAlign) -> Self {
        self.text_align = align;
        self
    }

    pub fn white_space(mut self, white_space: WhiteSpace) -> Self {
        self.white_space = white_space;
        self
    }
}

impl Component for Text {
    fn build(self) -> RenderResult {
        TextNode::new(self.text)
            .style(
                self.layout
                    .to_style()
                    .font_size(self.size)
                    .color(self.color)
                    .font(self.font)
                    .font_style(self.font_style)
                    .text_align(self.text_align)
                    .white_space(self.white_space),
            )
            .build()
    }
}

/// Low level design-system-agnostic text-rendering component
#[derive(Debug, Default)]
pub struct TextNode {
    render_node: Node,
}

impl TextNode {
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            render_node: Node {
                node_type: Some(NodeType {
                    node_type: Some(node_type::NodeType::Text(protos::spelldawn::Text {
                        label: text.into(),
                    })),
                }),
                ..Node::default()
            },
        }
    }
}

impl HasRenderNode for TextNode {
    fn render_node(&mut self) -> &mut Node {
        &mut self.render_node
    }
}

impl Component for TextNode {
    fn build(self) -> RenderResult {
        RenderResult::Node(Box::new(self.render_node))
    }
}
