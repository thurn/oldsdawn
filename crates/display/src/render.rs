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
use data::game::GameState;
use data::primitives::Side;
use protos::spelldawn::game_command::Command;

use crate::response_builder::ResponseBuilder;
use crate::{animations, sync};

pub fn connect(game: &GameState, user_side: Side) -> Result<Vec<Command>> {
    let mut builder = ResponseBuilder::new(user_side, false);
    sync::run(&mut builder, game)?;
    Ok(builder.commands)
}

pub fn render_updates(game: &GameState, user_side: Side) -> Result<Vec<Command>> {
    let mut builder = ResponseBuilder::new(user_side, true);

    for step in &game.updates.steps {
        sync::run(&mut builder, &step.snapshot)?;
        animations::render(&mut builder, &step.update, &step.snapshot)?;
    }

    // UpdateTracker does not contain the final state of the game
    sync::run(&mut builder, game)?;

    Ok(builder.commands)
}
