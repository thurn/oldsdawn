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

use std::collections::HashSet;

use data::card_state::CardState;
use data::primitives::Side;
use itertools::Itertools;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::game_object_identifier::Id;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{
    CommandList, GameCommand, GameObjectIdentifier, MoveGameObjectsCommand, ObjectPosition,
    RunInParallelCommand,
};

use crate::adapters;

/// Subset of [CommandPhase] during which moves can occur.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum MovePhase {
    StartMoves,
    StandardMoves,
}

/// Key used to sort [Command]s into distinct groups
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum CommandPhase {
    StartMoves,
    PreUpdate,
    Update,
    Animate,
    StandardMoves,
    RenderInterface,
    PostMove,
    End,
}

/// Options for configuring a move command.
#[derive(Clone, Copy, Debug)]
pub struct MoveType {
    /// Target phase to run the move command.
    ///
    /// Defaults to [MovePhase::StandardMoves].
    pub phase: MovePhase,
    /// Run in parallel with other moves during this phase.
    ///
    /// Defaults to true.
    pub parallel: bool,
    /// Whether this command can be skipped.
    ///
    /// If false, skip this move if the target object receives a required move
    /// command. Defaults to true.
    pub required: bool,
}

impl Default for MoveType {
    fn default() -> Self {
        Self { phase: MovePhase::StandardMoves, parallel: true, required: true }
    }
}

/// Keeps track of [Command]s required to update the client
#[derive(Clone, Debug)]
pub struct ResponseBuilder {
    pub user_side: Side,
    pub animate: bool,
    commands: Vec<(CommandPhase, Command)>,
    required: HashSet<Id>,
    moves: Vec<(Id, ObjectPosition, MoveType)>,
}

impl ResponseBuilder {
    pub fn new(user_side: Side, animate: bool) -> Self {
        Self { user_side, animate, commands: vec![], required: HashSet::new(), moves: vec![] }
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

    /// Move a GameObject to a new client position
    pub fn move_object(&mut self, id: Id, position: ObjectPosition, move_type: MoveType) {
        if move_type.required {
            self.required.insert(id);
        }
        self.moves.push((id, position, move_type))
    }

    /// Move a card to a new client position
    pub fn move_card(&mut self, card: &CardState, position: Position, move_type: MoveType) {
        self.move_object(
            Id::CardId(adapters::adapt_card_id(card.id)),
            ObjectPosition { sorting_key: card.sorting_key, position: Some(position) },
            move_type,
        )
    }

    /// Converts this builder into a [Command] vector
    pub fn build(mut self) -> Vec<Command> {
        self.moves.retain(|(id, _, move_type)| move_type.required || !self.required.contains(id));
        self.moves.sort_by_key(|(id, _, move_type)| (move_type.phase, *id));

        let mut moves = vec![];
        for (phase, commands) in &self.moves.iter().group_by(|(_, _, move_type)| move_type.phase) {
            moves.extend(self.create_move_group(phase, commands.collect()));
        }

        for (phase, command) in moves {
            self.push(phase, command);
        }

        self.commands.sort_by_key(|(phase, _)| *phase);
        self.commands.into_iter().map(|(_, c)| c).collect()
    }

    fn create_move_group(
        &self,
        move_phase: MovePhase,
        group: Vec<&(Id, ObjectPosition, MoveType)>,
    ) -> Vec<(CommandPhase, Command)> {
        let phase = to_command_phase(move_phase);
        let mut result = vec![];
        let mut parallel = vec![];
        for (id, position, move_type) in group {
            if move_type.parallel {
                parallel.push(self.new_move_command(*id, position.clone()))
            } else {
                result.push((phase, self.new_move_command(*id, position.clone())))
            }
        }

        result.push((
            phase,
            Command::RunInParallel(RunInParallelCommand {
                commands: parallel
                    .into_iter()
                    .map(|command| CommandList {
                        commands: vec![GameCommand { command: Some(command) }],
                    })
                    .collect(),
            }),
        ));
        result
    }

    fn new_move_command(&self, id: Id, position: ObjectPosition) -> Command {
        Command::MoveGameObjects(MoveGameObjectsCommand {
            ids: vec![GameObjectIdentifier { id: Some(id) }],
            position: Some(position),
            disable_animation: !self.animate,
        })
    }
}

fn to_command_phase(move_phase: MovePhase) -> CommandPhase {
    match move_phase {
        MovePhase::StartMoves => CommandPhase::StartMoves,
        MovePhase::StandardMoves => CommandPhase::StandardMoves,
    }
}
