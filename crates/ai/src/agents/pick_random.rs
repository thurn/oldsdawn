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

// Agent which selects actions at random

use anyhow::Result;
use data::game_actions::UserAction;
use data::primitives::Side;
use data::with_error::WithError;

use crate::core::legal_actions;
use crate::core::types::StatePredictionIterator;

pub fn execute(mut states: StatePredictionIterator, side: Side) -> Result<UserAction> {
    let mut game = states.next().with_error(|| "Expected at least one GameState")?.state;
    let collected = legal_actions::evaluate(&game, side).collect::<Vec<_>>();
    game.choose_randomly(collected.into_iter()).with_error(|| "Expected at least one action")
}
