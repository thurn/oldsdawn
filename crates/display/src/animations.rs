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

//! Functions for turning [GameUpdate]s into sequences of [GameCommand]s

use data::card_state::CardState;
use data::game::{GameState, RaidPhase};
use data::primitives::{CardId, RoomId, Side};
use data::updates::GameUpdate;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{
    game_object_identifier, DelayCommand, DisplayGameMessageCommand, GameCommand, GameMessageType,
    GameObjectIdentifier, InitiateRaidCommand, MoveGameObjectsCommand, ObjectPosition,
    ObjectPositionDeck, ObjectPositionStaging, PlayerName, TimeValue,
};
use ui::prompts;

use crate::full_sync::CardCreationStrategy;
use crate::{adapters, full_sync};

/// Takes a [GameUpdate] and converts it into an animation, a series of
/// corresponding [GameCommand]s. Commands are appended to the provided
/// `commands` list.
pub fn render(
    commands: &mut Vec<GameCommand>,
    update: GameUpdate,
    game: &GameState,
    user_side: Side,
) {
    match update {
        GameUpdate::StartTurn(side) => {
            start_turn(commands, side);
        }
        GameUpdate::DrawCard(card_id) | GameUpdate::MoveCard(card_id) => {
            move_card(commands, game, game.card(card_id), user_side);
        }
        GameUpdate::RevealCard(card_id) => {
            reveal_card(commands, game.card(card_id), user_side);
        }
        GameUpdate::InitiateRaid(room_id) => {
            initiate_raid(commands, game, room_id, user_side);
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
fn move_card(commands: &mut Vec<GameCommand>, game: &GameState, card: &CardState, user_side: Side) {
    push_optional(
        commands,
        full_sync::adapt_position(card, game.card(card.id).position, user_side).map(|position| {
            Command::MoveGameObjects(MoveGameObjectsCommand {
                ids: vec![adapt_game_object_id(card.id)],
                position: Some(position),
                disable_animation: false,
            })
        }),
    )
}

/// Commands to reveal the indicated card to all players
fn reveal_card(commands: &mut Vec<GameCommand>, card: &CardState, user_side: Side) {
    if user_side != card.side {
        push(
            commands,
            Command::MoveGameObjects(MoveGameObjectsCommand {
                ids: vec![adapt_game_object_id(card.id)],
                position: Some(ObjectPosition {
                    sorting_key: 0,
                    position: Some(Position::Staging(ObjectPositionStaging {})),
                }),
                disable_animation: false,
            }),
        );
        push(commands, delay(1500));
    }
}

/// Constructs a delay command
fn delay(milliseconds: u32) -> Command {
    Command::Delay(DelayCommand { duration: Some(TimeValue { milliseconds }) })
}

/// Starts the `side` player's turn
fn start_turn(commands: &mut Vec<GameCommand>, side: Side) {
    push(
        commands,
        Command::DisplayGameMessage(DisplayGameMessageCommand {
            message_type: match side {
                Side::Overlord => GameMessageType::Dusk.into(),
                Side::Champion => GameMessageType::Dawn.into(),
            },
        }),
    )
}

fn initiate_raid(
    commands: &mut Vec<GameCommand>,
    game: &GameState,
    target: RoomId,
    user_side: Side,
) {
    let raid = game.data.raid.expect("No raid is active");
    push(
        commands,
        Command::InitiateRaid(InitiateRaidCommand {
            initiator: adapters::to_player_name(Side::Champion, user_side).into(),
            room_id: adapters::adapt_room_id(target).into(),
        }),
    );

    if raid.phase == RaidPhase::Activation {
        match user_side {
            Side::Overlord => push(commands, prompts::activation_prompt()),
            Side::Champion => push(commands, prompts::waiting_prompt()),
        }
    }
}

/// Converts a [CardId] into a client [GameObjectIdentifier]
fn adapt_game_object_id(id: CardId) -> GameObjectIdentifier {
    GameObjectIdentifier {
        id: Some(game_object_identifier::Id::CardId(adapters::adapt_card_id(id))),
    }
}

/// Adds a command to `commands` if it is not `None`.
fn push_optional(commands: &mut Vec<GameCommand>, option: Option<Command>) {
    if let Some(command) = option {
        push(commands, command);
    }
}

/// Helper function to wrap a [Command] in [GameCommand] and add it to
/// `commands`
fn push(commands: &mut Vec<GameCommand>, command: Command) {
    commands.push(GameCommand { command: Some(command) })
}
