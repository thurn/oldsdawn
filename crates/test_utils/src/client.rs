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

//! A fake game client. Records server responses about a game and stores them in
//! [TestGame].

use std::cmp::Ordering;
use std::collections::HashMap;

use anyhow::{Context, Result};
use data::card_name::CardName;
use data::card_state::{CardData, CardPosition, CardState};
use data::game::GameState;
use data::primitives::{
    ActionCount, CardId, CardType, GameId, ManaValue, PointsValue, RoomId, Side, UserId,
};
use display::rendering;
use display::rendering::CardCreationStrategy;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{
    card_target, game_object_identifier, CardIdentifier, CardTarget, CardView, ClientRoomLocation,
    CreateOrUpdateCardCommand, GameAction, GameIdentifier, GameRequest, ObjectPosition,
    ObjectPositionDiscardPile, ObjectPositionHand, ObjectPositionRoom, PlayCardAction, PlayerName,
    PlayerView,
};
use server::database::Database;
use server::GameResponse;

/// A fake game client for testing.
///
/// This struct keeps track of server responses related to an ongoing game and
/// converts them into a useful format for writing tests. This enables our
/// 'black box' testing strategy, where the game is almost exclusively tested
/// via the public client-facing API.
#[derive(Debug, Clone)]
pub struct TestGame {
    pub data: ClientGameData,
    pub user: ClientPlayer,
    pub opponent: ClientPlayer,
    pub cards: ClientCards,
    game: GameState,
}

impl TestGame {
    /// Creates a new game, starting in the provided [GameState].
    ///
    /// It is usually better to create a blank new game and then update its
    /// state via the action methods on this struct instead of putting a bunch
    /// of information into the [GameState] here, because this helps avoid
    /// coupling tests to the specific implementation details of [GameState].
    pub fn new(game: GameState) -> Self {
        Self {
            data: ClientGameData::default(),
            user: ClientPlayer::new(PlayerName::User),
            opponent: ClientPlayer::new(PlayerName::Opponent),
            cards: ClientCards::default(),
            game,
        }
    }

    /// Execute a simulated client request for this game, updating the client
    /// state as appropriate based on the responses. Returns the [GameResponse]
    /// for this action or an error if the server request failed.
    pub fn perform_action(&mut self, action: Action, user_id: UserId) -> Result<GameResponse> {
        let response = server::handle_request(
            self,
            &GameRequest {
                action: Some(GameAction { action: Some(action) }),
                game_id: Some(GameIdentifier { value: crate::GAME_ID.value }),
                user_id: user_id.value,
            },
        )?;

        for command in &response.command_list.commands {
            self.handle_command(command.command.as_ref().with_context(|| "Command not received")?);
        }

        Ok(response)
    }

    /// Adds a named card to its owner's hand.
    ///
    /// This function operates by locating a test card in the owner's deck and
    /// overwriting it with the provided `card_name`. This card is then moved to
    /// the user's hand via [GameState::move_card].
    ///
    /// This function will *not* check the legality of drawing a card, invoke
    /// any game events, or append a game update, but it will correctly
    /// update the card's sorting key. Returns the client [CardIdentifier]
    /// for the drawn card.
    ///
    /// Panics if no test cards remain in the user's deck.
    pub fn add_to_hand(&mut self, card_name: CardName) -> CardIdentifier {
        let side = side_for_card_name(card_name);
        let card_id = self
            .game
            .cards_in_position(side, CardPosition::DeckUnknown(side))
            .find(|c| c.name.is_test_card())
            .expect("No test cards remaining in deck")
            .id;
        overwrite_card(&mut self.game, card_id, card_name);
        self.game.move_card(card_id, CardPosition::Hand(side));

        let command = rendering::create_or_update_card(
            &self.game,
            self.game.card(card_id),
            side,
            CardCreationStrategy::SnapToCurrentPosition,
        );
        self.handle_command(&command);
        rendering::adapt_card_id(card_id)
    }

    /// Draws and then plays a named card.
    ///
    /// This function first adds a copy of the requested card to the user's hand
    /// via [Self::add_to_hand]. The card is then played via the standard
    /// [PlayCardAction].
    ///
    /// If the card is a minion, project, scheme, or upgrade card, it is played
    /// into the [crate::ROOM_ID] room. The [GameResponse] produced by
    /// playing the card is returned.
    pub fn play_from_hand(&mut self, card_name: CardName) -> GameResponse {
        let card_id = self.add_to_hand(card_name);

        let target = match rules::get(card_name).card_type {
            CardType::Minion | CardType::Project | CardType::Scheme | CardType::Upgrade => {
                Some(CardTarget {
                    card_target: Some(card_target::CardTarget::RoomId(
                        rendering::adapt_room_id(crate::ROOM_ID).into(),
                    )),
                })
            }
            _ => None,
        };

        self.perform_action(
            Action::PlayCard(PlayCardAction { card_id: Some(card_id), target }),
            crate::USER_ID,
        )
        .expect("Server error playing card")
    }

    /// Returns a vec containing the titles of all of the cards in the provided
    /// player's hand, or [crate::HIDDEN_CARD] if the card's title is
    /// unknown. Titles will be ordered by their sorting key.
    pub fn hand(&self, player: PlayerName) -> Vec<String> {
        self.cards.names_in_position(Position::Hand(ObjectPositionHand { owner: player.into() }))
    }

    /// Returns a player's discard pile in the same manner as [Self::hand]
    pub fn discard_pile(&self, player: PlayerName) -> Vec<String> {
        self.cards.names_in_position(Position::DiscardPile(ObjectPositionDiscardPile {
            owner: player.into(),
        }))
    }

    /// Returns a vector containing the card titles in the provided `location`
    /// of a given room, Titles are structured in the same manner described
    /// in [Self::hand].
    pub fn room_cards(&self, room_id: RoomId, location: ClientRoomLocation) -> Vec<String> {
        self.cards.names_in_position(Position::Room(ObjectPositionRoom {
            room_id: rendering::adapt_room_id(room_id).into(),
            room_location: location.into(),
        }))
    }

    fn handle_command(&mut self, command: &Command) {
        self.data.update(command.clone());
        self.user.update(command.clone());
        self.opponent.update(command.clone());
        self.cards.update(command.clone());
    }
}

/// Overwrites the card with ID `card_id` in `game` to be a new card with the
/// provided `card_name`.
pub fn overwrite_card(game: &mut GameState, card_id: CardId, card_name: CardName) {
    *game.card_mut(card_id) = CardState {
        id: card_id,
        name: card_name,
        side: card_id.side,
        position: game.card(card_id).position,
        sorting_key: 0,
        data: CardData::default(),
    };
}

pub fn side_for_card_name(name: CardName) -> Side {
    rules::get(name).side
}

impl Database for TestGame {
    fn generate_game_id(&self) -> Result<GameId> {
        Ok(crate::GAME_ID)
    }

    fn game(&self, _id: GameId) -> Result<GameState> {
        Ok(self.game.clone())
    }

    fn write_game(&mut self, game: &GameState) -> Result<()> {
        self.game = game.clone();
        Ok(())
    }
}

/// Simulated game state in an ongoing [TestGame]
#[derive(Debug, Clone, Default)]
pub struct ClientGameData {
    priority: Option<PlayerName>,
}

impl ClientGameData {
    pub fn priority(&self) -> PlayerName {
        self.priority.unwrap()
    }

    fn update(&mut self, command: Command) {
        if let Command::UpdateGameView(update_game) = command {
            self.priority =
                PlayerName::from_i32(update_game.game.as_ref().unwrap().current_priority)
        }
    }
}

/// Simulated player state in an ongoing [TestGame]
#[derive(Debug, Clone)]
pub struct ClientPlayer {
    name: PlayerName,
    mana: Option<ManaValue>,
    actions: Option<ActionCount>,
    score: Option<PointsValue>,
}

impl ClientPlayer {
    fn new(name: PlayerName) -> Self {
        Self { name, mana: None, actions: None, score: None }
    }

    pub fn mana(&self) -> ManaValue {
        self.mana.expect("Mana")
    }

    pub fn actions(&self) -> ActionCount {
        self.actions.expect("Actions")
    }

    pub fn score(&self) -> PointsValue {
        self.score.expect("Points")
    }

    fn update(&mut self, command: Command) {
        if let Command::UpdateGameView(update) = command {
            self.update_with_player(if self.name == PlayerName::User {
                update.game.unwrap().user.unwrap()
            } else {
                update.game.unwrap().opponent.unwrap()
            });
        }
    }

    fn update_with_player(&mut self, player: PlayerView) {
        self.mana = Some(player.mana.unwrap().amount);
        self.actions = Some(player.action_tracker.unwrap().available_action_count);
        self.score = Some(player.score.unwrap().score)
    }
}

/// Simulated card state in an ongoing [TestGame]
#[derive(Debug, Clone, Default)]
pub struct ClientCards {
    cards: HashMap<CardId, ClientCard>,
}

impl ClientCards {
    /// Returns an iterator over the cards in a given [Position] in an arbitrary
    /// order.
    pub fn in_position(&self, position: Position) -> impl Iterator<Item = &ClientCard> {
        self.cards.values().filter(move |c| c.position() == position)
    }

    /// Returns a list of the titles of cards in the provided `position`, or the
    /// string [crate::HIDDEN_CARD] if no title is available. Cards are
    /// sorted in position order based on their `sorting_key` with ties being
    /// broken arbitrarily.
    pub fn names_in_position(&self, position: Position) -> Vec<String> {
        let mut result = self
            .in_position(position)
            .map(|c| c.title_option().unwrap_or_else(|| crate::HIDDEN_CARD.to_string()))
            .collect::<Vec<_>>();
        result.sort();
        result
    }

    fn update(&mut self, command: Command) {
        match command {
            Command::CreateOrUpdateCard(create_or_update) => {
                let card_view = create_or_update.clone().card.expect("CardView");
                let card_id = server::to_server_card_id(&card_view.card_id).expect("CardId");
                self.cards
                    .entry(card_id)
                    .and_modify(|c| c.view = Some(card_view))
                    .or_insert_with(|| ClientCard::new(create_or_update));
            }
            Command::MoveGameObjects(move_objects) => {
                let position = move_objects.clone().position.expect("ObjectPosition");
                for id in move_objects.ids {
                    if let game_object_identifier::Id::CardId(identifier) = id.id.expect("ID") {
                        let card_id = server::to_server_card_id(&Some(identifier)).expect("CardId");
                        assert!(
                            self.cards.contains_key(&card_id),
                            "Expected a CreateOrUpdate command before a Move command for card {:?}",
                            card_id
                        );
                        let mut card = self.cards.get_mut(&card_id).unwrap();
                        card.position = Some(position.clone());
                    }
                }
            }
            Command::DestroyCard(destroy_card) => {
                let card_id = server::to_server_card_id(&destroy_card.card_id).expect("CardId");
                self.cards.remove(&card_id);
            }
            _ => {}
        }
    }
}

/// Simulated state of a specific card
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClientCard {
    view: Option<CardView>,
    position: Option<ObjectPosition>,
}

impl ClientCard {
    /// Returns the game object position for this card
    pub fn position(&self) -> Position {
        self.position.clone().expect("CardPosition").position.expect("Position")
    }

    /// Returns the user-visible title for this card. Panics if no title is
    /// available.
    pub fn title(&self) -> String {
        self.title_option().expect("No card title found")
    }

    /// Returns the user-visible title for this card, if one is available
    pub fn title_option(&self) -> Option<String> {
        Some(self.view.clone()?.revealed_card?.title?.text)
    }

    fn new(command: CreateOrUpdateCardCommand) -> Self {
        Self { view: command.card, position: command.create_position }
    }
}

impl PartialOrd for ClientCard {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.position.as_ref()?.sorting_key.partial_cmp(&other.position.as_ref()?.sorting_key)
    }
}
