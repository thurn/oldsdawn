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

use anyhow::Result;
use with_error::WithError;

/// Helper which keeps track of an evaluator score and the action that produced
/// it.
pub struct ScoredAction<T> {
    score: i64,
    action: Option<T>,
}

impl<T> ScoredAction<T>
where
    T: Copy,
{
    pub fn new(score: i64) -> Self {
        Self { score, action: None }
    }

    pub fn action(&self) -> Result<T> {
        self.action.with_error(|| "Expected action")
    }

    pub fn score(&self) -> i64 {
        self.score
    }

    /// Insert this action & score if they are greater than the current score.
    pub fn insert_max(&mut self, action: T, score: i64) {
        if score > self.score {
            self.score = score;
            self.action = Some(action);
        }
    }

    /// Insert this action & score if they are lower than the current score.
    pub fn insert_min(&mut self, action: T, score: i64) {
        if score < self.score {
            self.score = score;
            self.action = Some(action);
        }
    }
}
