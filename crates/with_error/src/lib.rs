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

//! Error handlers which can be configured to panic on error

use std::convert::Infallible;
use std::error;
use std::fmt::Display;

use anyhow::{Context, Error};

/// Should the system panic when an error is encountered?
///
/// (this used to be a cfg value, but changing that makes build times much
/// longer)
pub const ERROR_PANIC: bool = false;

/// Wrapper around [anyhow::ensure] which can be configured to panic on error.
#[macro_export]
macro_rules! verify {
    ($($tts:tt)*) => {
        if with_error::ERROR_PANIC {
            assert!($($tts)*);
        } else {
            use anyhow::ensure;
            ensure!($($tts)*);
        }
    }
}

/// Wrapper around [anyhow::bail] which can be configured to panic on error.
#[macro_export]
macro_rules! fail {
    ($($tts:tt)*) => {
        if with_error::ERROR_PANIC {
            panic!($($tts)*);
        } else {
            use anyhow::bail;
            bail!($($tts)*);
        }
    }
}

pub trait WithError<T, E> {
    /// Wrapper around anyhow::with_context. Wraps the error value with
    /// additional context that is evaluated lazily only once an error does
    /// occur. Can be configured to panic on error.
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
        if ERROR_PANIC {
            self.with_context(|| {
                panic!("Error: {}", context());
                ""
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
        if ERROR_PANIC {
            self.with_context(|| {
                panic!("Error: {}", context());
                ""
            })
        } else {
            self.with_context(context)
        }
    }
}
