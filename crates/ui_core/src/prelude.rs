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

//! Re-exports commonly-used types for UI rendering, intended to be used via
//! wildcard import.

pub use crate::component::{Buildable, Component, RenderResult};
pub use crate::flexbox::{Column, HasRenderNode, Row};
pub use crate::style::{DimensionExt, Edge, Layout, Style};
