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

//! Functions for turning [GameUpdate]s into animations via a sequence of
//! [GameCommand]s.
//!
//! Animations must be non-essential to the interface state since any changes
//! they make are transient on the client and will be lost if the client
//! reconnects. Non-decorative changes to the client state should be handled by
//! the [full_sync] module.

use data::card_state::CardState;
use data::game::GameState;
use data::primitives::{CardId, RoomId, Side};
use data::updates::GameUpdate;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::object_position::Position;
#[allow(unused)] // Used in rustdoc
use protos::spelldawn::{
    game_object_identifier, DelayCommand, DisplayGameMessageCommand, GameCommand, GameMessageType,
    GameObjectIdentifier, MoveGameObjectsCommand, ObjectPosition, ObjectPositionDeck,
    ObjectPositionStaging, PlayerName, RoomVisitType, TimeValue, VisitRoomCommand,
};

use crate::full_sync::CardCreationStrategy;
use crate::response_builder::{CommandPhase, ResponseBuilder};
use crate::{adapters, full_sync};

/// Takes a [GameUpdate] and converts it into an animation, a series of
/// corresponding [GameCommand]s. Commands are appended to the provided
/// `commands` list.
pub fn render(commands: &mut ResponseBuilder, update: GameUpdate, game: &GameState) {
    match update {
        GameUpdate::StartTurn(side) => {
            start_turn(commands, side);
        }
        GameUpdate::DrawCard(card_id) | GameUpdate::MoveCard(card_id) => {
            move_card(commands, game.card(card_id));
        }
        GameUpdate::RevealCard(card_id) => {
            reveal_card(commands, game, game.card(card_id));
        }
        GameUpdate::InitiateRaid(room_id) => {
            initiate_raid(commands, room_id);
        }
        _ => {}
    }
}

/// Builds a [CardCreationStrategy] for representing the provided `card_id`
/// being drawn.
pub fn card_draw_creation_strategy(user_side: Side, card_id: CardId) -> CardCreationStrategy {
    if card_id.side == user_side {
        CardCreationStrategy::DrawUserCard
    } else {
        CardCreationStrategy::CreateAtPosition(ObjectPosition {
            sorting_key: u32::MAX,
            position: Some(Position::Deck(ObjectPositionDeck {
                owner: PlayerName::Opponent.into(),
            })),
        })
    }
}

/// Appends a move card command to move a card to its current location. Skips
/// appending the command if the destination would not be a valid game position,
/// e.g. if it is [CardPosition::DeckUnknown].
fn move_card(commands: &mut ResponseBuilder, card: &CardState) {
    commands.push_optional(
        CommandPhase::Animate,
        full_sync::adapt_position(card, commands.user_side).map(|position| {
            Command::MoveGameObjects(MoveGameObjectsCommand {
                ids: vec![adapt_game_object_id(card.id)],
                position: Some(position),
                disable_animation: false,
            })
        }),
    )
}

/// Commands to reveal the indicated card to all players
fn reveal_card(commands: &mut ResponseBuilder, game: &GameState, card: &CardState) {
    if commands.user_side != card.side
        && game.data.raid.map_or(true, |raid| !card.is_in_room(raid.target))
    {
        // If the hidden card is not part of an active raid, animate it to
        // the staging area on reveal.
        commands.push(
            CommandPhase::Animate,
            Command::MoveGameObjects(MoveGameObjectsCommand {
                ids: vec![adapt_game_object_id(card.id)],
                position: Some(ObjectPosition {
                    sorting_key: 0,
                    position: Some(Position::Staging(ObjectPositionStaging {})),
                }),
                disable_animation: false,
            }),
        );
        commands.push(CommandPhase::Animate, delay(1500));
    }
}

/// Constructs a delay command
fn delay(milliseconds: u32) -> Command {
    Command::Delay(DelayCommand { duration: Some(TimeValue { milliseconds }) })
}

/// Starts the `side` player's turn
fn start_turn(commands: &mut ResponseBuilder, side: Side) {
    commands.push(
        CommandPhase::Animate,
        Command::DisplayGameMessage(DisplayGameMessageCommand {
            message_type: match side {
                Side::Overlord => GameMessageType::Dusk.into(),
                Side::Champion => GameMessageType::Dawn.into(),
            },
        }),
    )
}

fn initiate_raid(commands: &mut ResponseBuilder, target: RoomId) {
    if commands.user_side == Side::Overlord {
        commands.push(
            CommandPhase::PreUpdate,
            Command::VisitRoom(VisitRoomCommand {
                initiator: adapters::to_player_name(Side::Champion, commands.user_side).into(),
                room_id: adapters::adapt_room_id(target).into(),
                visit_type: RoomVisitType::InitiateRaid.into(),
            }),
        );
        commands.push(CommandPhase::PreUpdate, delay(500));
    }
}

/// Converts a [CardId] into a client [GameObjectIdentifier]
fn adapt_game_object_id(id: CardId) -> GameObjectIdentifier {
    GameObjectIdentifier {
        id: Some(game_object_identifier::Id::CardId(adapters::adapt_card_id(id))),
    }
}
