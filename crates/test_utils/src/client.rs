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
//! [TestSession].

use std::cmp::Ordering;
use std::collections::HashMap;

use adapters;
use ai::core::legal_actions;
use anyhow::Result;
use data::card_name::CardName;
use data::card_state::{CardPosition, CardState};
use data::game::GameState;
use data::game_actions::UserAction;
use data::player_name::PlayerId;
use data::primitives::{
    ActionCount, CardId, CardType, GameId, ManaValue, PointsValue, RoomId, Side,
};
use protos::spelldawn::card_targeting::Targeting;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::game_object_identifier::Id;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{
    card_target, node_type, ArrowTargetRoom, CardAnchorNode, CardIdentifier, CardTarget, CardView,
    ClientItemLocation, ClientRoomLocation, CommandList, EventHandlers, GameAction,
    GameMessageType, GameObjectIdentifier, GameRequest, InitiateRaidAction, NoTargeting, Node,
    NodeType, ObjectPosition, ObjectPositionBrowser, ObjectPositionDiscardPile, ObjectPositionHand,
    ObjectPositionItem, ObjectPositionRevealedCards, ObjectPositionRoom, PlayCardAction,
    PlayInRoom, PlayerName, PlayerView, RevealedCardView, RevealedCardsBrowserSize, RoomIdentifier,
};
use rules::dispatch;
use server::requests;
use server::requests::GameResponse;
use with_error::WithError;

use crate::fake_database::FakeDatabase;
use crate::{fake_database, ROOM_ID};

/// A helper for interacting with a database and server calls during testing.
///
/// This struct keeps track of server responses and converts them into a useful
/// format for writing tests. This enables our 'black box' testing strategy,
/// where the game is almost exclusively tested via the public client-facing
/// API.
///
/// There are actually two perspectives on an ongoing game: each player has
/// their own view of the state of the game, which differs due to hidden
/// information. This struct has two different [TestClient]s which get updated
/// based on server responses, representing what the two players are seeing.
#[derive(Clone)]
pub struct TestSession {
    /// This is the perspective of the player identified by the `user_id`
    /// parameter to [Self::new].
    pub user: TestClient,
    /// This is the perspective of the player identified by the `opponent_id`
    /// parameter to [Self::new].
    pub opponent: TestClient,
    database: FakeDatabase,
}

impl TestSession {
    /// Creates a new game, starting in the provided [GameState].
    ///
    /// It is usually better to create a blank new game and then update its
    /// state via the action methods on this struct instead of putting a bunch
    /// of information into the [GameState] here, because this helps avoid
    /// coupling tests to the specific implementation details of [GameState].
    pub fn new(database: FakeDatabase, user_id: PlayerId, opponent_id: PlayerId) -> Self {
        Self { user: TestClient::new(user_id), opponent: TestClient::new(opponent_id), database }
    }

    pub fn game_id(&self) -> GameId {
        self.database.game().id
    }

    pub fn user_id(&self) -> PlayerId {
        self.user.id
    }

    pub fn opponent_id(&self) -> PlayerId {
        self.opponent.id
    }

    /// Returns the user player state for the user client, (i.e. the user's
    /// state from *their own* perspective).
    pub fn me(&self) -> &ClientPlayer {
        &self.user.this_player
    }

    /// Returns the opponent player state for the opponent client (i.e. the
    /// opponent's state from their perspective).
    pub fn you(&self) -> &ClientPlayer {
        &self.opponent.this_player
    }

    /// Simulates a client connecting to the server.
    ///
    /// Returns the commands which would be sent to the client when connected.
    pub fn connect(&mut self, user_id: PlayerId) -> Result<CommandList> {
        let result = requests::handle_connect(&mut self.database, user_id)?;
        let to_update = match () {
            _ if user_id == self.user.id => &mut self.user,
            _ if user_id == self.opponent.id => &mut self.opponent,
            _ => panic!("Unknown user id: {:?}", user_id),
        };

        // Clear all previous state
        *to_update = TestClient::new(user_id);

        for command in result.commands.iter() {
            let c = command.command.as_ref().with_error(|| "command")?;
            to_update.handle_command(c);
        }

        Ok(result)
    }

    /// Execute a simulated client request for this game as a specific user,
    /// updating the client state as appropriate based on the responses.
    /// Returns the [GameResponse] for this action or an error if the server
    /// request failed.
    pub fn perform_action(&mut self, action: Action, player_id: PlayerId) -> Result<GameResponse> {
        let response = requests::handle_request(
            &mut self.database,
            &GameRequest {
                action: Some(GameAction { action: Some(action) }),
                player_id: Some(fake_database::to_player_identifier(player_id)),
            },
        )?;

        let (opponent_id, local, remote) = self.opponent_local_remote(player_id);
        for command in &response.command_list.commands {
            local.handle_command(command.command.as_ref().expect("Empty command"));
        }

        if let Some((channel_user_id, list)) = &response.opponent_response {
            assert_eq!(*channel_user_id, opponent_id);
            for command in &list.commands {
                remote.handle_command(command.command.as_ref().expect("Empty command"));
            }
        }

        Ok(response)
    }

    /// Equivalent function to [Self::perform_action] which does not return the
    /// action result.
    pub fn perform(&mut self, action: Action, user_id: PlayerId) {
        self.perform_action(action, user_id).expect("Request failed");
    }

    /// Helper function to invoke [Self::perform] to initiate a raid on the
    /// provided `room_id`.
    pub fn initiate_raid(&mut self, room_id: RoomId) -> GameResponse {
        self.perform_action(
            Action::InitiateRaid(InitiateRaidAction {
                room_id: adapters::room_identifier(room_id),
            }),
            self.player_id_for_side(Side::Champion),
        )
        .expect("Server Error")
    }

    /// Adds a named card to its owner's hand.
    ///
    /// This function operates by locating a test card in the owner's deck and
    /// overwriting it with the provided `card_name`. This card is then
    /// moved to the user's hand via [GameState::move_card_internal]. The
    /// complete game state is synced for both players by invoking
    /// [Self::connect].
    ///
    /// This function will *not* spend action points, check the legality of
    /// drawing a card, invoke any game events, or append a game update. It
    /// will correctly update the card's sorting key, however.
    ///
    /// Returns the client [CardIdentifier] for the drawn card. Panics if no
    /// test cards remain in the user's deck.
    pub fn add_to_hand(&mut self, card_name: CardName) -> CardIdentifier {
        let side = side_for_card_name(card_name);
        let card_id = self
            .database
            .game()
            .cards_in_position(side, CardPosition::DeckUnknown(side))
            .filter(|c| c.name.is_test_card())
            .last() // Use last to avoid overwriting 'next draw' configuration
            .unwrap()
            .id;
        overwrite_card(self.database.game_mut(), card_id, card_name);
        self.database.game_mut().move_card_internal(card_id, CardPosition::Hand(side));
        self.database.game_mut().card_mut(card_id).set_revealed_to(card_id.side, true);

        self.connect(self.user.id).expect("User connection error");
        self.connect(self.opponent.id).expect("Opponent connection error");

        adapters::card_identifier(card_id)
    }

    /// Creates and then plays a named card as the user who owns this card.
    ///
    /// This function first adds a copy of the requested card to the user's hand
    /// via [Self::add_to_hand]. The card is then played via the standard
    /// [PlayCardAction]. Action points and mana must be available and are spent
    /// as normal.
    ///
    /// If the card is a minion, project, or scheme card, it is played
    /// into the [crate::ROOM_ID] room. The [CardIdentifier] for the played card
    /// is returned.
    ///
    /// Panics if the server returns an error for playing this card.
    pub fn play_from_hand(&mut self, card_name: CardName) -> CardIdentifier {
        self.play_impl(
            card_name,
            match rules::get(card_name).card_type {
                CardType::Minion | CardType::Project | CardType::Scheme => Some(ROOM_ID),
                _ => None,
            },
        )
    }

    /// Equivalent method to [Self::play_from_hand] which specifies
    /// a target room to use.
    pub fn play_with_target_room(
        &mut self,
        card_name: CardName,
        room_id: RoomId,
    ) -> CardIdentifier {
        self.play_impl(card_name, Some(room_id))
    }

    fn play_impl(&mut self, card_name: CardName, room_id: Option<RoomId>) -> CardIdentifier {
        let card_id = self.add_to_hand(card_name);
        let target = room_id.map(|room_id| CardTarget {
            card_target: Some(card_target::CardTarget::RoomId(adapters::room_identifier(room_id))),
        });

        self.play_card(
            card_id,
            self.database.game().player(side_for_card_name(card_name)).id,
            target,
        );

        card_id
    }

    /// Helper to take the [PlayCardAction] with a given card ID.
    pub fn play_card(
        &mut self,
        card_id: CardIdentifier,
        player_id: PlayerId,
        target: Option<CardTarget>,
    ) {
        self.perform(
            Action::PlayCard(PlayCardAction { card_id: Some(card_id), target }),
            player_id,
        );
    }

    /// Locate a button containing the provided `text` in the provided player's
    /// interface controls and invoke its registered action.
    pub fn click_on(&mut self, player_id: PlayerId, text: impl Into<String>) -> GameResponse {
        let (_, player, _) = self.opponent_local_remote(player_id);
        let handlers = player.interface.controls().find_handlers(text);
        let action = handlers.expect("Button not found").on_click.expect("OnClick not found");
        self.perform_action(action.action.expect("Action"), player_id).expect("Server Error")
    }

    /// Returns true if the last-received Game Message was 'Dawn'.
    pub fn dawn(&self) -> bool {
        assert_eq!(self.user.data.last_message(), self.opponent.data.last_message());
        self.user.data.last_message() == GameMessageType::Dawn
    }

    /// Returns true if the last-received Game Message was 'Dusk'.
    pub fn dusk(&self) -> bool {
        assert_eq!(self.user.data.last_message(), self.opponent.data.last_message());
        self.user.data.last_message() == GameMessageType::Dusk
    }

    /// Returns true if the last-received Game Messages indicated the `winner`
    /// player won the game
    pub fn is_victory_for_player(&self, winner: Side) -> bool {
        self.player_for_side(winner).data.last_message() == GameMessageType::Victory
            && self.player_for_side(winner.opponent()).data.last_message()
                == GameMessageType::Defeat
    }

    /// Returns the [TestClient] for a given player in the game.
    pub fn player(&self, player_id: PlayerId) -> &TestClient {
        match () {
            _ if player_id == self.user.id => &self.user,
            _ if player_id == self.opponent.id => &self.opponent,
            _ => panic!("Unknown player id: {:?}", player_id),
        }
    }

    /// Returns the [TestClient] for the [Side] player in the game.
    pub fn player_for_side(&self, side: Side) -> &TestClient {
        self.player(self.player_id_for_side(side))
    }

    /// Looks up the [PlayerId] for the [Side] player.
    pub fn player_id_for_side(&self, side: Side) -> PlayerId {
        if self.database.game().player(side).id == self.user.id {
            self.user.id
        } else if self.database.game().player(side).id == self.opponent.id {
            self.opponent.id
        } else {
            panic!("Cannot find PlayerId for side {:?}", side)
        }
    }

    /// Activates an ability of a card owned by the user based on its ability
    /// index.
    pub fn activate_ability(&mut self, card_id: CardIdentifier, index: u32) {
        self.activate_ability_impl(card_id, index, None)
    }

    /// Activates an ability of a card with a target room
    pub fn activate_ability_with_target(
        &mut self,
        card_id: CardIdentifier,
        index: u32,
        target: RoomId,
    ) {
        self.activate_ability_impl(card_id, index, Some(target))
    }

    /// Evaluates legal actions for the [Side] player in the current game state.
    pub fn legal_actions(&self, side: Side) -> Vec<UserAction> {
        legal_actions::evaluate(self.database.game.as_ref().expect("game"), side)
            .expect("Error evaluating legal actions")
            .collect()
    }

    fn activate_ability_impl(
        &mut self,
        card_id: CardIdentifier,
        index: u32,
        target: Option<RoomId>,
    ) {
        self.perform(
            Action::PlayCard(PlayCardAction {
                card_id: Some(CardIdentifier { ability_id: Some(index), ..card_id }),
                target: target.map(|room_id| CardTarget {
                    card_target: Some(card_target::CardTarget::RoomId(adapters::room_identifier(
                        room_id,
                    ))),
                }),
            }),
            self.user_id(),
        );
    }

    /// Returns a triple of (opponent_id, local_client, remote_client) for the
    /// provided player ID
    fn opponent_local_remote(
        &mut self,
        player_id: PlayerId,
    ) -> (PlayerId, &mut TestClient, &mut TestClient) {
        match () {
            _ if player_id == self.user.id => {
                (self.opponent.id, &mut self.user, &mut self.opponent)
            }
            _ if player_id == self.opponent.id => {
                (self.user.id, &mut self.opponent, &mut self.user)
            }
            _ => panic!("Unknown user id: {:?}", player_id),
        }
    }
}

/// Overwrites the card with ID `card_id` in `game` to be a new card with the
/// provided `card_name`.
pub fn overwrite_card(game: &mut GameState, card_id: CardId, card_name: CardName) {
    let card = game.card(card_id);
    let mut state = CardState::new(card_id, card_name, false);
    state.set_position_internal(card.sorting_key, card.position());
    *game.card_mut(card_id) = state;

    // Our delegate cache logic assumes the set of card names in a game will not
    // change while the game is in progress, so we need to delete the cache.
    dispatch::populate_delegate_cache(game);
}

/// Returns the [Side] player who owns the [CardName] card
fn side_for_card_name(name: CardName) -> Side {
    rules::get(name).side
}

/// Represents a user client connected to a test game
#[derive(Clone)]
pub struct TestClient {
    pub id: PlayerId,
    pub data: ClientGameData,
    /// A player's view of *their own* player state.
    pub this_player: ClientPlayer,
    /// A player's view of *their opponent's* player state.
    pub other_player: ClientPlayer,
    pub interface: ClientInterface,
    pub cards: ClientCards,
    pub history: Vec<Command>,
}

impl TestClient {
    fn new(id: PlayerId) -> Self {
        Self {
            id,
            data: ClientGameData::default(),
            this_player: ClientPlayer::new(PlayerName::User),
            other_player: ClientPlayer::new(PlayerName::Opponent),
            interface: ClientInterface::default(),
            cards: ClientCards { player_id: id, card_map: HashMap::default() },
            history: vec![],
        }
    }

    pub fn get_card(&self, id: CardIdentifier) -> &ClientCard {
        self.cards.get(id)
    }

    fn handle_command(&mut self, command: &Command) {
        self.data.update(command.clone());
        self.this_player.update(command.clone());
        self.other_player.update(command.clone());
        self.interface.update(command.clone());
        self.cards.update(command.clone());
        self.history.push(command.clone());
    }
}

/// Simulated game state in an ongoing [TestSession]
#[derive(Clone, Default)]
pub struct ClientGameData {
    raid_active: Option<bool>,
    object_positions: HashMap<GameObjectIdentifier, ObjectPosition>,
    last_message: Option<GameMessageType>,
}

impl ClientGameData {
    pub fn raid_active(&self) -> bool {
        self.raid_active.expect("raid_active")
    }

    /// Returns the position of the `id` object along with its index within its
    /// position list
    pub fn object_index_position(&self, id: Id) -> (u32, Position) {
        let position = self
            .object_positions
            .get(&GameObjectIdentifier { id: Some(id) })
            .unwrap_or_else(|| panic!("No position available for {:?}", id))
            .clone()
            .position
            .expect("position");
        let mut positions = self
            .object_positions
            .iter()
            .filter(|(_, p)| p.position.as_ref().expect("position") == &position)
            .collect::<Vec<_>>();
        positions.sort_by_key(|(_, p)| (p.sorting_key, p.sorting_subkey));
        let index = positions
            .iter()
            .position(|(object_id, _)| object_id.id.as_ref().expect("id") == &id)
            .expect("index");

        (index as u32, position)
    }

    /// Returns the position of the `id` object
    pub fn object_position(&self, id: Id) -> Position {
        self.object_index_position(id).1
    }

    /// Returns the last-seen `GameMessage`.
    pub fn last_message(&self) -> GameMessageType {
        self.last_message.expect("Game Message")
    }

    fn update(&mut self, command: Command) {
        match command {
            Command::UpdateGameView(update_game) => {
                let game = update_game.game.as_ref().unwrap();
                self.raid_active = Some(game.raid_active);
                for card in &game.cards {
                    self.object_positions
                        .insert(card_object_id(card.card_id), card.card_position.clone().unwrap());
                }

                let non_card = game.game_object_positions.as_ref().unwrap();
                self.insert_position(deck_id(PlayerName::User), &non_card.user_deck);
                self.insert_position(deck_id(PlayerName::Opponent), &non_card.opponent_deck);
                self.insert_position(identity_id(PlayerName::User), &non_card.user_identity);
                self.insert_position(
                    identity_id(PlayerName::Opponent),
                    &non_card.opponent_identity,
                );
                self.insert_position(discard_id(PlayerName::User), &non_card.user_discard);
                self.insert_position(discard_id(PlayerName::Opponent), &non_card.opponent_deck);
            }
            Command::MoveGameObjects(move_objects) => {
                for move_object in move_objects.moves {
                    let p = move_object.position.as_ref().expect("ObjectPosition").clone();
                    self.object_positions.insert(move_object.id.expect("id"), p);
                }
            }
            Command::DisplayGameMessage(display_message) => {
                self.last_message = GameMessageType::from_i32(display_message.message_type);
            }
            Command::CreateTokenCard(create_token) => {
                let card = create_token.card.as_ref().expect("card");
                self.object_positions.insert(
                    card_object_id(card.card_id),
                    card.card_position.clone().expect("position"),
                );
            }
            _ => {}
        }
    }

    fn insert_position(&mut self, id: GameObjectIdentifier, position: &Option<ObjectPosition>) {
        self.object_positions.insert(id, position.clone().expect("position"));
    }
}

fn card_object_id(id: Option<CardIdentifier>) -> GameObjectIdentifier {
    GameObjectIdentifier { id: Some(Id::CardId(id.expect("card_id"))) }
}

fn deck_id(name: PlayerName) -> GameObjectIdentifier {
    GameObjectIdentifier { id: Some(Id::Deck(name as i32)) }
}

fn identity_id(name: PlayerName) -> GameObjectIdentifier {
    GameObjectIdentifier { id: Some(Id::Identity(name as i32)) }
}

fn discard_id(name: PlayerName) -> GameObjectIdentifier {
    GameObjectIdentifier { id: Some(Id::DiscardPile(name as i32)) }
}

/// Simulated player state in an ongoing [TestSession]
#[derive(Debug, Clone)]
pub struct ClientPlayer {
    name: PlayerName,
    mana: Option<ManaValue>,
    bonus_mana: Option<ManaValue>,
    actions: Option<ActionCount>,
    score: Option<PointsValue>,
    can_take_action: Option<bool>,
}

impl ClientPlayer {
    fn new(name: PlayerName) -> Self {
        Self {
            name,
            mana: None,
            bonus_mana: None,
            actions: None,
            score: None,
            can_take_action: None,
        }
    }

    pub fn mana(&self) -> ManaValue {
        self.mana.expect("Mana")
    }

    pub fn bonus_mana(&self) -> ManaValue {
        self.bonus_mana.expect("BonusMana")
    }

    pub fn actions(&self) -> ActionCount {
        self.actions.expect("Actions")
    }

    pub fn score(&self) -> PointsValue {
        self.score.expect("Points")
    }

    pub fn can_take_action(&self) -> bool {
        self.can_take_action.expect("can_take_action")
    }

    fn update(&mut self, command: Command) {
        if let Command::UpdateGameView(update) = command {
            self.update_with_player(if self.name == PlayerName::User {
                update.game.unwrap().user
            } else {
                update.game.unwrap().opponent
            });
        }
    }

    fn update_with_player(&mut self, player: Option<PlayerView>) {
        if let Some(p) = player {
            self.mana = Some(p.mana.clone().expect("mana").base_mana);
            self.bonus_mana = Some(p.mana.clone().expect("mana").bonus_mana);
            self.actions = Some(p.action_tracker.clone().expect("actions").available_action_count);
            self.score = Some(p.score.clone().expect("score").score);
            self.can_take_action = Some(p.can_take_action);
        }
    }
}

/// Simulated user interface state
#[derive(Debug, Clone, Default)]
pub struct ClientInterface {
    main_controls: Option<Node>,
    card_anchors: Vec<CardAnchorNode>,
}

impl ClientInterface {
    pub fn main_controls_option(&self) -> Option<Node> {
        self.main_controls.clone()
    }

    pub fn main_controls(&self) -> &Node {
        self.main_controls.as_ref().expect("MainControls Node")
    }

    pub fn card_anchors(&self) -> &Vec<CardAnchorNode> {
        &self.card_anchors
    }

    pub fn controls(&self) -> Vec<&Node> {
        let mut result =
            vec![self.main_controls.as_ref()].into_iter().flatten().collect::<Vec<_>>();
        result.extend(self.card_anchor_nodes());
        result
    }

    pub fn card_anchor_nodes(&self) -> Vec<&Node> {
        self.card_anchors().iter().filter_map(|node| node.node.as_ref()).collect()
    }

    fn update(&mut self, command: Command) {
        if let Command::UpdateGameView(update) = command {
            let controls = update.game.as_ref().expect("game").main_controls.as_ref();
            self.main_controls = controls.and_then(|c| c.node.clone());
            self.card_anchors = controls.map_or(vec![], |c| c.card_anchor_nodes.clone());
        }
    }
}

pub trait HasText {
    /// Returns true if there are any text nodes contained within this tree
    /// which contain the provided string.    
    fn has_text(&self, text: impl Into<String>) -> bool;

    /// Populates `path` with the series of nodes leading to the node which
    /// contains the provided text. Leaves `path` unchanged if no node
    /// containing this text is found.
    fn find_text(&self, path: &mut Vec<Node>, text: impl Into<String>);

    /// Finds the path to the provided `text` via [Self::find_text] and then
    /// searches up the path for registered [EventHandlers].
    fn find_handlers(&self, text: impl Into<String>) -> Option<EventHandlers>;

    /// Returns all text contained within this tree
    fn get_text(&self) -> Vec<String>;
}

impl HasText for Node {
    fn has_text(&self, text: impl Into<String>) -> bool {
        let string = text.into();
        if let Some(NodeType { node_type: Some(node_type::NodeType::Text(s)) }) = &self.node_type {
            if s.label.contains(string.as_str()) {
                return true;
            }
        }

        for child in &self.children {
            if child.has_text(string.as_str()) {
                return true;
            }
        }

        false
    }

    fn find_text(&self, path: &mut Vec<Node>, text: impl Into<String>) {
        let string = text.into();
        if self.has_text(string.as_str()) {
            path.push(self.clone());
        }

        for child in &self.children {
            child.find_text(path, string.as_str());
        }
    }

    fn find_handlers(&self, text: impl Into<String>) -> Option<EventHandlers> {
        let mut nodes = vec![];
        self.find_text(&mut nodes, text);
        nodes.reverse();
        nodes.iter().find_map(|node| node.event_handlers.clone())
    }

    fn get_text(&self) -> Vec<String> {
        let mut result = vec![];
        if let Some(NodeType { node_type: Some(node_type::NodeType::Text(s)) }) = &self.node_type {
            result.push(s.label.clone())
        }

        for child in &self.children {
            result.extend(child.get_text());
        }

        result
    }
}

impl HasText for Vec<&Node> {
    fn has_text(&self, text: impl Into<String>) -> bool {
        let string = text.into();
        for node in self {
            if node.has_text(string.as_str()) {
                return true;
            }
        }
        false
    }

    fn find_text(&self, path: &mut Vec<Node>, text: impl Into<String>) {
        let string = text.into();
        for node in self {
            if node.has_text(string.as_str()) {
                return node.find_text(path, string.as_str());
            }
        }
    }

    fn find_handlers(&self, text: impl Into<String>) -> Option<EventHandlers> {
        let string = text.into();
        for node in self {
            if let Some(handlers) = node.find_handlers(string.as_str()) {
                return Some(handlers);
            }
        }
        None
    }

    fn get_text(&self) -> Vec<String> {
        let mut result = vec![];
        for node in self {
            result.extend(node.get_text());
        }
        result
    }
}

/// Simulated card state in an ongoing [TestSession]
#[derive(Debug, Clone)]
pub struct ClientCards {
    pub player_id: PlayerId,
    pub card_map: HashMap<CardIdentifier, ClientCard>,
}

impl ClientCards {
    pub fn get(&self, card_id: CardIdentifier) -> &ClientCard {
        self.card_map.get(&card_id).unwrap_or_else(|| panic!("Card not found: {:?}", card_id))
    }

    /// Returns a vec containing the titles of all of the cards in the provided
    /// player's hand from the perspective of the this client, or
    /// [crate::HIDDEN_CARD] if the card's title is unknown. Titles will be
    /// ordered by their sorting key.
    pub fn hand(&self, player: PlayerName) -> Vec<String> {
        self.names_in_position(Position::Hand(ObjectPositionHand { owner: player.into() }))
    }

    /// Returns a vec of card names currently displayed in the card browser
    pub fn browser(&self) -> Vec<String> {
        self.names_in_position(Position::Browser(ObjectPositionBrowser {}))
    }

    /// Returns a vec of card names currently displayed in the revealed cards
    /// area
    pub fn revealed_cards(&self) -> Vec<String> {
        let mut result = self.names_in_position(Position::Revealed(ObjectPositionRevealedCards {
            size: RevealedCardsBrowserSize::Small as i32,
        }));
        result.append(&mut self.names_in_position(Position::Revealed(
            ObjectPositionRevealedCards { size: RevealedCardsBrowserSize::Large as i32 },
        )));
        result
    }

    /// Returns a player's discard pile in the same manner as [Self::hand]
    pub fn discard_pile(&self, player: PlayerName) -> Vec<String> {
        self.names_in_position(Position::DiscardPile(ObjectPositionDiscardPile {
            owner: player.into(),
        }))
    }

    /// Returns left items in play
    pub fn left_items(&self) -> Vec<String> {
        self.names_in_position(Position::Item(ObjectPositionItem {
            item_location: ClientItemLocation::Left.into(),
        }))
    }

    /// Returns left items in play
    pub fn right_items(&self) -> Vec<String> {
        self.names_in_position(Position::Item(ObjectPositionItem {
            item_location: ClientItemLocation::Right as i32,
        }))
    }

    /// Returns a vector containing the card titles in the provided `location`
    /// of a given room, Titles are structured in the same manner described
    /// in [Self::hand].
    pub fn room_cards(&self, room_id: RoomId, location: ClientRoomLocation) -> Vec<String> {
        self.names_in_position(Position::Room(ObjectPositionRoom {
            room_id: adapters::room_identifier(room_id),
            room_location: location.into(),
        }))
    }

    /// Returns an iterator over the cards in a given [Position] in an arbitrary
    /// order.
    pub fn in_position(&self, position: Position) -> impl Iterator<Item = &ClientCard> {
        self.card_map.values().filter(move |c| c.position() == position)
    }

    /// Iterator over cards in a player's hand
    pub fn cards_in_hand(&self, player: PlayerName) -> impl Iterator<Item = &ClientCard> {
        self.in_position(Position::Hand(ObjectPositionHand { owner: player.into() }))
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
            Command::UpdateGameView(update_game) => {
                let game = update_game.game.as_ref().unwrap();
                self.card_map.clear();
                for card in &game.cards {
                    self.card_map.insert(card.card_id.expect("card_id"), ClientCard::new(card));
                }
            }
            Command::MoveGameObjects(move_objects) => {
                for move_object in move_objects.moves {
                    let p = move_object.position.as_ref().expect("ObjectPosition").clone();
                    let id = match move_object.id.expect("id").id.expect("id") {
                        Id::CardId(identifier) => identifier,
                        _ => panic!("Expected CardId"),
                    };
                    self.card_map.get_mut(&id).unwrap().set_position(p);
                }
            }
            Command::CreateTokenCard(create_token) => {
                let card = create_token.card.as_ref().expect("card");
                self.card_map.insert(card.card_id.expect("card_id"), ClientCard::new(card));
            }
            _ => {}
        }
    }
}

/// Simulated state of a specific card
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ClientCard {
    id: Option<CardIdentifier>,
    title: Option<String>,
    position: Option<ObjectPosition>,
    revealed_to_me: Option<bool>,
    is_face_up: Option<bool>,
    can_play: Option<bool>,
    valid_rooms: Option<Vec<RoomIdentifier>>,
    arena_icon: Option<String>,
    top_left_icon: Option<String>,
    top_right_icon: Option<String>,
    bottom_left_icon: Option<String>,
    bottom_right_icon: Option<String>,
}

impl ClientCard {
    pub fn id(&self) -> CardIdentifier {
        self.id.expect("card_id")
    }

    /// Returns the game object position for this card
    pub fn position(&self) -> Position {
        self.position.clone().expect("CardPosition").position.expect("Position")
    }

    /// Returns the user-visible title for this card. Panics if no title is
    /// available.
    pub fn title(&self) -> String {
        self.title_option().expect("No card title found")
    }

    /// Returns a copy of the user-visible title for this card, if one is
    /// available
    pub fn title_option(&self) -> Option<String> {
        self.title.clone()
    }

    pub fn revealed_to_me(&self) -> bool {
        self.revealed_to_me.expect("revealed_to_me")
    }

    pub fn is_face_up(&self) -> bool {
        self.is_face_up.expect("is_face_up")
    }

    pub fn can_play(&self) -> bool {
        self.can_play.expect("can_play")
    }

    pub fn valid_rooms(&self) -> Vec<RoomIdentifier> {
        self.valid_rooms.as_ref().expect("valid_rooms").clone()
    }

    pub fn arena_icon(&self) -> String {
        self.arena_icon.clone().expect("arena_icon")
    }

    pub fn top_left_icon(&self) -> String {
        self.top_left_icon.clone().expect("top_left_icon")
    }

    pub fn top_right_icon(&self) -> String {
        self.top_right_icon.clone().expect("top_right_icon")
    }

    pub fn bottom_left_icon(&self) -> String {
        self.bottom_left_icon.clone().expect("bottom_left_icon")
    }

    pub fn bottom_right_icon(&self) -> String {
        self.bottom_right_icon.clone().expect("bottom_right_icon")
    }

    pub fn set_position(&mut self, position: ObjectPosition) {
        self.position = Some(position);
    }

    fn new(view: &CardView) -> Self {
        let mut result = Self::default();
        result.update(view);
        result
    }

    fn update(&mut self, view: &CardView) {
        self.id = view.card_id;
        self.position = view.card_position.clone();
        self.revealed_to_me = Some(view.revealed_to_viewer);
        self.is_face_up = Some(view.is_face_up);
        if let Some(revealed) = &view.revealed_card {
            self.update_revealed_card(revealed);
        }

        self.arena_icon = card_icon(view, |v| v.card_icons?.arena_icon?.text);
        self.top_left_icon = card_icon(view, |v| v.card_icons?.top_left_icon?.text);
        self.top_right_icon = card_icon(view, |v| v.card_icons?.top_right_icon?.text);
        self.bottom_left_icon = card_icon(view, |v| v.card_icons?.bottom_left_icon?.text);
        self.bottom_right_icon = card_icon(view, |v| v.card_icons?.bottom_right_icon?.text);
    }

    fn update_revealed_card(&mut self, revealed: &RevealedCardView) {
        let targets = {
            || {
                Some(match revealed.targeting.as_ref()?.targeting.as_ref()? {
                    Targeting::NoTargeting(NoTargeting { can_play }) => (*can_play, vec![]),
                    Targeting::PlayInRoom(PlayInRoom { valid_rooms }) => {
                        (!valid_rooms.is_empty(), valid_rooms.clone())
                    }
                    Targeting::ArrowTargetRoom(ArrowTargetRoom { valid_rooms, .. }) => {
                        (!valid_rooms.is_empty(), valid_rooms.clone())
                    }
                })
            }
        }();
        if let Some((can_play, valid_rooms)) = targets {
            self.can_play = Some(can_play);
            self.valid_rooms =
                Some(valid_rooms.iter().map(|i| RoomIdentifier::from_i32(*i).unwrap()).collect())
        }

        if let Some(title) = revealed.clone().title.map(|title| title.text) {
            self.title = Some(title);
        }
    }
}

fn card_icon(view: &CardView, function: impl Fn(CardView) -> Option<String>) -> Option<String> {
    function(view.clone())
}

impl PartialOrd for ClientCard {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.position.as_ref()?.sorting_key.partial_cmp(&other.position.as_ref()?.sorting_key)
    }
}
