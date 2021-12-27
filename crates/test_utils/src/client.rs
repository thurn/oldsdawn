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
use data::card_name::CardName;
use data::card_state::{CardData, CardPosition, CardState};
use data::game::{GameState};
use data::primitives::{
    ActionCount, GameId, ManaValue, PointsValue, RoomId, Side, UserId,
};
use display::rendering;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{
    CardIdentifier,
    GameAction, GameIdentifier, GameRequest, PlayerName, PlayerView,
};
use rules::mutations;
use server::database::Database;

#[derive(Debug, Clone)]
pub struct TestGame {
    pub user_side: Side,
    pub data: ClientGameData,
    pub user: ClientPlayer,
    pub opponent: ClientPlayer,
    pub cards: ClientCards,
    game: GameState,
}

impl TestGame {
    /// The [UserId] for the user who the test is running as
    pub const USER_ID: UserId = UserId { value: 1 };
    /// The [UserId] for the user who is *not* running the test
    pub const OPPONENT_ID: UserId = UserId { value: 2 };
    /// The standard [GameId] used for this game
    pub const GAME_ID: GameId = GameId { value: 1 };
    /// [RoomId] used by default for targeting
    pub const ROOM_ID: RoomId = RoomId::RoomA;

    pub fn new(game: GameState, user_side: Side) -> Self {
        let (_user, _opponent) = match user_side {
            Side::Overlord => (&game.overlord, &game.champion),
            Side::Champion => (&game.champion, &game.overlord),
        };

        Self {
            user_side,
            data: ClientGameData::default(),
            user: ClientPlayer::new(PlayerName::User),
            opponent: ClientPlayer::new(PlayerName::Opponent),
            cards: ClientCards::default(),
            game,
        }
    }

    /// Execute a simulated client request for this game, updating the client state as appropriate
    /// based on the responses. Returns a vector of the received commands.
    pub fn perform_action(&mut self, action: Action) -> Vec<Command> {
        let commands = server::handle_request(
            self,
            &GameRequest {
                action: Some(GameAction { action: Some(action) }),
                game_id: Some(GameIdentifier { value: Self::GAME_ID.value }),
                user_id: Self::USER_ID.value,
            },
        )
        .expect("Server request failed")
        .command_list
        .commands
        .into_iter()
        .map(|c| c.command.expect("Empty command received"))
        .collect::<Vec<_>>();

        for command in &commands {
            self.data.update(command.clone());
            self.user.update(command.clone());
            self.opponent.update(command.clone());
            self.cards.update(command.clone());
        }

        commands
    }

    /// Adds a named card to the user's hand.
    ///
    /// This function operates by locating a test card in the user's deck and overwriting its state
    /// to a default [CardState] pointing to the provided [CardName] instead. This card is then
    /// moved to the user's hand via [mutations::move_card], which *will* invoke game events & game
    /// updates for a card being moved as normal. Returns the client [CardIdentifier] for the drawn
    /// card.
    pub fn draw_named_card(&mut self, card_name: CardName) -> CardIdentifier {
        let test_card = match self.user_side {
            Side::Overlord => CardName::TestOverlordSpell,
            Side::Champion => CardName::TestChampionSpell,
        };
        let deck_position = CardPosition::DeckUnknown(self.user_side);
        let card_id = self
            .game
            .cards_in_position(self.user_side, deck_position)
            .filter(|c| c.name == test_card)
            .next()
            .expect("No test cards remaining in deck")
            .id;
        *self.game.card_mut(card_id) = CardState {
            id: card_id,
            name: card_name,
            side: self.user_side,
            position: deck_position,
            sorting_key: 0,
            data: CardData::default(),
        };

        mutations::move_card(&mut self.game, card_id, CardPosition::Hand(self.user_side));

        rendering::adapt_card_id(card_id)
    }
}

impl Database for TestGame {
    fn generate_game_id(&self) -> Result<GameId> {
        Ok(Self::GAME_ID)
    }

    fn game(&self, _id: GameId) -> Result<GameState> {
        Ok(self.game.clone())
    }

    fn write_game(&mut self, game: &GameState) -> Result<()> {
        self.game = game.clone();
        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct ClientGameData {
    priority: Option<PlayerName>,
}

impl ClientGameData {
    pub fn priority(&self) -> PlayerName {
        self.priority.unwrap()
    }

    fn update(&mut self, command: Command) {
        match command {
            Command::UpdateGameView(update_game) => {
                self.priority =
                    PlayerName::from_i32(update_game.game.as_ref().unwrap().current_priority)
            }
            _ => {}
        }
    }
}

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
        self.mana.unwrap()
    }

    pub fn actions(&self) -> ActionCount {
        self.actions.unwrap()
    }

    pub fn score(&self) -> PointsValue {
        self.score.unwrap()
    }

    fn update(&mut self, command: Command) {
        match command {
            Command::UpdateGameView(update) => {
                self.update_with_player(if self.name == PlayerName::User {
                    update.game.unwrap().user.unwrap()
                } else {
                    update.game.unwrap().opponent.unwrap()
                });
            }
            _ => {}
        }
    }

    fn update_with_player(&mut self, player: PlayerView) {
        self.mana = Some(player.mana.unwrap().amount);
        self.actions = Some(player.action_tracker.unwrap().available_action_count);
        self.score = Some(player.score.unwrap().score)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ClientCards {}

impl ClientCards {
    fn update(&mut self, _command: Command) {}
}

#[derive(Debug, Clone, Default)]
pub struct ClientCard {}

impl ClientCard {
    pub fn new(_card: &CardState) -> Self {
        Self {}
    }

    fn update(&mut self, _command: &Command) {}
}
