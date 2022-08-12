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
    score: i32,
    action: Option<T>,
}

impl<T> ScoredAction<T>
where
    T: Copy,
{
    pub fn new(score: i32) -> Self {
        Self { score, action: None }
    }

    pub fn has_action(&self) -> bool {
        self.action.is_some()
    }

    pub fn action(&self) -> Result<T> {
        self.action.with_error(|| "No action found for ScoredAction")
    }

    pub fn score(&self) -> i32 {
        self.score
    }

    /// Returns this ScoredAction, populating `action` as its action if there is
    /// not currently a saved action.
    pub fn with_fallback_action(self, action: T) -> Self {
        if self.action.is_none() {
            Self { action: Some(action), ..self }
        } else {
            self
        }
    }

    /// Insert this action & score if they are greater than the current score.
    pub fn insert_max(&mut self, action: T, score: i32) {
        if !self.has_action() || score > self.score {
            self.score = score;
            self.action = Some(action);
        }
    }

    /// Insert this action & score if they are lower than the current score.
    pub fn insert_min(&mut self, action: T, score: i32) {
        if !self.has_action() || score < self.score {
            self.score = score;
            self.action = Some(action);
        }
    }
}
