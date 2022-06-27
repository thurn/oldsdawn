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

use protos::spelldawn::Node;

/// A component is any reusable piece of UI.
///
/// Typically this is a struct that implements `Debug` and has one or more
/// properties settable via a builder pattern.
///
/// Components can either return another component, typically by invoking its
/// `build` method, or can create a UI node directly, as discussed in
/// [RenderResult].
pub trait Component: Debug + ComponentExt {
    fn render(self) -> RenderResult;
}

/// Return type of the `render` function. There are three types of components:
///   * Wrapper components, which return a single child component
///   * Container components, which create a UI node and have a list of child
///     components
///   * Leaf components, which create a UI node without children
/// Typically you invoke the `build` method instead of creating this type
/// directly.
pub enum RenderResult {
    Component(Box<dyn Component>),
    Container(Box<Node>, Vec<Box<dyn Component>>),
    Node(Box<Node>),
}

/// Helper trait to let components be moved into a `Box`.
pub trait ComponentExt {
    fn render_boxed(self: Box<Self>) -> RenderResult;
}

impl<T: Component> ComponentExt for T {
    fn render_boxed(self: Box<Self>) -> RenderResult {
        self.render()
    }
}

/// Implements a `build` method for all components, allowing them to be moved
/// into a Box via a nice chainable syntax.
pub trait Buildable: Component + Sized + 'static {
    fn build(self) -> RenderResult {
        RenderResult::Component(Box::new(self))
    }
}

impl<T: Component + 'static> Buildable for T {}
