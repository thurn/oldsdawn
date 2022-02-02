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

use data::primitives::Side;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::game_object_identifier::Id;
use protos::spelldawn::{
    CommandList, GameCommand, GameObjectIdentifier, MoveGameObjectsCommand, ObjectPosition,
    RunInParallelCommand,
};

/// Key used to sort [Command]s into distinct groups
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum CommandPhase {
    PreUpdate,
    Update,
    Animate,
    Move,
    RenderInterface,
    PostMove,
}

/// Keeps track of [Command]s required to update the client
pub struct ResponseBuilder {
    pub user_side: Side,
    pub animate: bool,
    commands: Vec<(CommandPhase, Command)>,
    moves: Vec<(Id, ObjectPosition)>,
}

impl ResponseBuilder {
    pub fn new(user_side: Side, animate: bool) -> Self {
        Self { user_side, animate, commands: vec![], moves: vec![] }
    }

    /// Append a new command to this builder
    pub fn push(&mut self, phase: CommandPhase, command: Command) {
        self.commands.push((phase, command))
    }

    pub fn push_optional(&mut self, phase: CommandPhase, option: Option<Command>) {
        if let Some(command) = option {
            self.push(phase, command);
        }
    }

    pub fn push_all(&mut self, phase: CommandPhase, iterator: impl Iterator<Item = Command>) {
        for item in iterator {
            self.push(phase, item)
        }
    }

    /// Appends a command to move `id` to a new [ObjectPosition].
    ///
    /// All commands added in this manner will be run in parallel during the
    /// move phase.
    pub fn move_object(&mut self, id: Id, position: ObjectPosition) {
        self.moves.push((id, position));
    }

    /// Converts this builder into a [Command] vector
    pub fn build(mut self) -> Vec<Command> {
        if !self.moves.is_empty() {
            self.moves.sort_by_key(|(id, _)| *id);
            let parallel_move = Command::RunInParallel(RunInParallelCommand {
                commands: self
                    .moves
                    .into_iter()
                    .map(|(id, position)| CommandList {
                        commands: vec![GameCommand {
                            command: Some(Command::MoveGameObjects(MoveGameObjectsCommand {
                                ids: vec![GameObjectIdentifier { id: Some(id) }],
                                position: Some(position),
                                disable_animation: !self.animate,
                            })),
                        }],
                    })
                    .collect(),
            });
            self.commands.push((CommandPhase::Move, parallel_move));
        }

        self.commands.sort_by_key(|(phase, _)| *phase);
        self.commands.into_iter().map(|(_, c)| c).collect()
    }
}
