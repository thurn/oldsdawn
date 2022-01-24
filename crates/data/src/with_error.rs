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

use std::convert::Infallible;
use std::error;
use std::fmt::Display;

use anyhow::{Context, Error};

pub trait WithError<T, E> {
    /// Wrapper around anyhow::with_context. Wraps the error value with
    /// additional context that is evaluated lazily only once an error does
    /// occur. Panics in debug builds.
    fn with_error<C, F>(self, f: F) -> Result<T, Error>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T> WithError<T, Infallible> for Option<T> {
    fn with_error<C, F>(self, context: F) -> Result<T, Error>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        #[allow(unreachable_code)]
        if cfg!(debug_assertions) {
            self.with_context(|| {
                panic!("Error: {}", context());
                "" // TODO: figure out why this is needed
            })
        } else {
            self.with_context(context)
        }
    }
}

impl<T, E> WithError<T, E> for Result<T, E>
where
    E: error::Error + Send + Sync + 'static,
{
    fn with_error<C, F>(self, context: F) -> Result<T, Error>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        #[allow(unreachable_code)]
        if cfg!(debug_assertions) {
            self.with_context(|| {
                panic!("Error: {}", context());
                "" // TODO: figure out why this is needed
            })
        } else {
            self.with_context(context)
        }
    }
}
