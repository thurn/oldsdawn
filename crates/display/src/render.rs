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

use adapters::response_builder::{ResponseBuilder, ResponseState};
use anyhow::Result;
use data::game::GameState;
use data::primitives::Side;
use protos::spelldawn::game_command::Command;

use crate::{animations, sync};

pub fn connect(game: &GameState, user_side: Side) -> Result<Vec<Command>> {
    let mut builder =
        ResponseBuilder::new(user_side, ResponseState { animate: false, is_final_update: true });
    sync::run(&mut builder, game)?;
    Ok(builder.commands)
}

pub fn render_updates(game: &GameState, user_side: Side) -> Result<Vec<Command>> {
    let mut builder =
        ResponseBuilder::new(user_side, ResponseState { animate: true, is_final_update: false });

    for step in &game.updates.steps {
        sync::run(&mut builder, &step.snapshot)?;
        animations::render(&mut builder, &step.update, &step.snapshot)?;
    }

    builder.state.is_final_update = true;
    sync::run(&mut builder, game)?;

    Ok(builder.commands)
}
