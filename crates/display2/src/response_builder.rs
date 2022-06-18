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

use std::collections::HashMap;

use anyhow::Result;
use bitflags::bitflags;
use data::card_state::CardState;
use data::primitives::{CardId, Side};
use data::with_error::WithError;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::game_object_identifier::Id;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{
    CardIdentifier, CommandList, GameCommand, GameObjectIdentifier, MoveGameObjectsCommand,
    ObjectPosition, RunInParallelCommand,
};

use crate::adapters;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum UpdateType {
    /// Marker for updates which have not previously been registered.
    None,

    /// Updates to game state, interface state, and card state which are checked
    /// as part of every game update diff
    General,

    /// Most types of standard card movements, e.g. move zone, shuffle into
    /// deck, draw card, destroy card
    Utility,

    /// Update to reveal a card to the opponent.
    Reveal,

    /// Animations for specialized game events, e.g. start turn, initiate raid,
    /// level up room, score card
    Animation,
}

#[derive(Clone, Debug, Default)]
pub struct CardUpdateTypes {
    data: HashMap<CardIdentifier, UpdateType>,
}

impl CardUpdateTypes {
    pub fn insert(&mut self, id: CardId, update_type: UpdateType) {
        let identifier = adapters::adapt_card_id(id);
        match self.data.get(&identifier) {
            None => {
                self.data.insert(identifier, update_type);
            }
            Some(u) if update_type > *u => {
                self.data.insert(identifier, update_type);
            }
            _ => {}
        };
    }

    pub fn get(&self, id: CardIdentifier) -> UpdateType {
        *self.data.get(&id).unwrap_or(&UpdateType::None)
    }
}

bitflags! {
    pub struct ResponseOptions: u32 {
        const ANIMATE = 0b00000001;
        const IS_INITIAL_CONNECT = 0b00000010;
    }
}

/// Keeps track of [Command]s required to update the client
#[derive(Clone, Debug)]
pub struct ResponseBuilder {
    pub user_side: Side,
    pub options: ResponseOptions,
    card_update_types: CardUpdateTypes,
    commands: Vec<Command>,
    moves: Vec<(UpdateType, Id, ObjectPosition)>,
}

impl ResponseBuilder {
    pub fn new(
        user_side: Side,
        card_update_types: CardUpdateTypes,
        options: ResponseOptions,
    ) -> Self {
        Self { user_side, options, commands: vec![], card_update_types, moves: vec![] }
    }

    /// Append a new command to this builder
    pub fn push(&mut self, update_type: UpdateType, command: Command) {
        if let Some(card_id) = card_id_for_command(&command) {
            if update_type >= self.card_update_types.get(card_id) {
                self.commands.push(command)
            }
        } else {
            self.commands.push(command)
        }
    }

    pub fn push_optional(&mut self, update_type: UpdateType, option: Option<Command>) {
        if let Some(command) = option {
            self.push(update_type, command);
        }
    }

    pub fn push_all(&mut self, update_type: UpdateType, iterator: impl Iterator<Item = Command>) {
        for item in iterator {
            self.push(update_type, item)
        }
    }

    /// Move a GameObject to a new client position
    pub fn move_object(&mut self, update_type: UpdateType, id: Id, position: ObjectPosition) {
        self.moves.push((update_type, id, position));
    }

    /// Immediately move a GameObject to a new position
    pub fn move_object_immediate(
        &mut self,
        update_type: UpdateType,
        id: Id,
        position: ObjectPosition,
    ) {
        self.push(
            update_type,
            Command::MoveGameObjects(MoveGameObjectsCommand {
                ids: vec![GameObjectIdentifier { id: Some(id) }],
                position: Some(position),
                disable_animation: !self.options.contains(ResponseOptions::ANIMATE),
            }),
        );
    }

    /// Equivalent method to [Self::move_object] which takes an
    /// `Option<ObjectPosition>`.
    pub fn move_object_optional(
        &mut self,
        update_type: UpdateType,
        id: Id,
        position: Option<ObjectPosition>,
    ) {
        if let Some(p) = position {
            self.move_object(update_type, id, p)
        }
    }

    /// Move a card to a new client position
    pub fn move_card(&mut self, update_type: UpdateType, card: &CardState, position: Position) {
        self.move_object(
            update_type,
            Id::CardId(adapters::adapt_card_id(card.id)),
            ObjectPosition { sorting_key: card.sorting_key, position: Some(position), ..ObjectPosition::default() },
        )
    }

    pub fn move_card_immediate(
        &mut self,
        update_type: UpdateType,
        card: &CardState,
        position: Position,
    ) {
        self.push(
            update_type,
            Command::MoveGameObjects(MoveGameObjectsCommand {
                ids: vec![GameObjectIdentifier {
                    id: Some(Id::CardId(adapters::adapt_card_id(card.id))),
                }],
                position: Some(ObjectPosition {
                    sorting_key: card.sorting_key,
                    position: Some(position),
                    ..ObjectPosition::default()
                }),
                disable_animation: !self.options.contains(ResponseOptions::ANIMATE),
            }),
        );
    }

    pub fn apply_parallel_moves(&mut self) -> Result<()> {
        if !self.moves.is_empty() {
            self.moves.sort_by_key(|(_, id, _)| *id);
            let commands = self
                .moves
                .iter()
                .filter_map(|(update_type, id, position)| {
                    self.process_move(*update_type, *id, position)
                })
                .collect::<Vec<_>>();

            match commands.len() {
                0 => {}
                1 => self
                    .commands
                    .push(commands.into_iter().next().with_error(|| "Command expected")?),
                _ => self.commands.push(Command::RunInParallel(RunInParallelCommand {
                    commands: commands
                        .into_iter()
                        .map(|c| CommandList { commands: vec![GameCommand { command: Some(c) }] })
                        .collect(),
                })),
            }

            self.moves.clear();
        }

        Ok(())
    }

    pub fn adapt_player_name(&self, side: Side) -> i32 {
        adapters::to_player_name(side, self.user_side).into()
    }

    /// Converts this builder into a [Command] vector
    pub fn build(mut self) -> Result<Vec<Command>> {
        self.apply_parallel_moves()?;
        Ok(self.commands)
    }

    fn process_move(
        &self,
        update_type: UpdateType,
        id: Id,
        position: &ObjectPosition,
    ) -> Option<Command> {
        let include = if let Id::CardId(card_id) = id {
            update_type >= self.card_update_types.get(card_id)
        } else {
            true
        };

        if include {
            Some(Command::MoveGameObjects(MoveGameObjectsCommand {
                ids: vec![GameObjectIdentifier { id: Some(id) }],
                position: Some(position.clone()),
                disable_animation: !self.options.contains(ResponseOptions::ANIMATE),
            }))
        } else {
            None
        }
    }
}

fn card_id_for_command(command: &Command) -> Option<CardIdentifier> {
    match command {
        Command::CreateOrUpdateCard(c) => c.card.as_ref()?.card_id,
        Command::DestroyCard(c) => c.card_id,
        Command::MoveGameObjects(c) => {
            if let Id::CardId(card_id) = c.ids[0].id? {
                Some(card_id)
            } else {
                None
            }
        }
        _ => None,
    }
}
