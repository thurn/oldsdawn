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

//! Macros to assist with UI rendering

use protos::spelldawn::Node;

use crate::core::Component;

pub trait Rendered {
    fn to_node(self) -> Option<Node>;
}

impl<T: Component> Rendered for T {
    fn to_node(self) -> Option<Node> {
        Some(self.render())
    }
}

impl<T: Component> Rendered for Option<T> {
    fn to_node(self) -> Option<Node> {
        self.map(Component::render)
    }
}

impl Rendered for Node {
    fn to_node(self) -> Option<Node> {
        Some(self)
    }
}

impl Rendered for Option<Node> {
    fn to_node(self) -> Option<Node> {
        self
    }
}

/// Macro which converts its arguments into a vector of `Option<Node>`.
#[macro_export]
macro_rules! children {
    ($($x:expr),*) => {
        vec! [$(crate::macros::Rendered::to_node($x)),*]
    };
    ($($x:expr,)*) => {children!($($x),*)}
}

pub use children;
