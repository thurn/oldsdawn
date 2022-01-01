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

use data::card_state::CardState;
use data::game::GameState;
use data::primitives::{CardId, Side};
use data::updates::GameUpdate;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{
    game_object_identifier, DelayCommand, DisplayGameMessageCommand, GameCommand, GameMessageType,
    GameObjectIdentifier, MoveGameObjectsCommand, ObjectPosition, ObjectPositionDeck,
    ObjectPositionStaging, PlayerName, TimeValue,
};

use crate::full_sync;
use crate::full_sync::CardCreationStrategy;

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
        GameUpdate::DrawCard(card_id) => {
            draw_card(commands, game, game.card(card_id), user_side);
        }
        GameUpdate::MoveCard(card_id) => {
            move_card(commands, game, game.card(card_id), user_side);
        }
        GameUpdate::RevealCard(card_id) => {
            reveal_card(commands, game.card(card_id), user_side);
        }
        _ => {}
    }
}

/// Builds commands to represent a card being drawn
fn draw_card(commands: &mut Vec<GameCommand>, game: &GameState, card: &CardState, user_side: Side) {
    push(
        commands,
        Command::CreateOrUpdateCard(full_sync::create_or_update_card(
            game,
            card,
            user_side,
            if card.side == user_side {
                CardCreationStrategy::DrawUserCard
            } else {
                CardCreationStrategy::CreateAtPosition(ObjectPosition {
                    sorting_key: u32::MAX,
                    position: Some(Position::Deck(ObjectPositionDeck {
                        owner: PlayerName::Opponent.into(),
                    })),
                })
            },
        )),
    );

    move_card(commands, game, card, user_side);
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

/// Converts a [CardId] into a client [GameObjectIdentifier]
fn adapt_game_object_id(id: CardId) -> GameObjectIdentifier {
    GameObjectIdentifier {
        id: Some(game_object_identifier::Id::CardId(full_sync::adapt_card_id(id))),
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
