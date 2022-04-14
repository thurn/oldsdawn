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

use anyhow::{Context, Result};
use data::card_name::CardName;
use data::card_state::{CardPosition, CardState};
use data::game::GameState;
use data::primitives::{
    ActionCount, CardId, CardType, GameId, ManaValue, PlayerId, PointsValue, RoomId, Side,
};
use display::adapters;
use protos::spelldawn::card_targeting::Targeting;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::game_object_identifier::Id;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{
    card_target, game_object_identifier, node_type, CardAnchorNode, CardIdentifier, CardTarget,
    CardView, ClientRoomLocation, CommandList, CreateOrUpdateCardCommand, EventHandlers,
    GameAction, GameIdentifier, GameMessageType, GameObjectIdentifier, GameRequest,
    InitiateRaidAction, NoTargeting, Node, NodeType, ObjectPosition, ObjectPositionBrowser,
    ObjectPositionDiscardPile, ObjectPositionHand, ObjectPositionRoom, PlayCardAction, PlayerName,
    PlayerView, RevealedCardView, RoomTargeting,
};
use server::GameResponse;

use crate::fake_database::FakeDatabase;

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

    /// Returns the opponent player state for the oppponent client (i.e. the
    /// opponent's state from their perspective).
    pub fn you(&self) -> &ClientPlayer {
        &self.opponent.this_player
    }

    /// Simulates a client connecting to the server, either creating a new game
    /// or connecting to an existing game. Returns the commands which would
    /// be sent to the client when connected. If a new game is created, its
    /// ID will be 0.
    pub fn connect(&mut self, user_id: PlayerId, game_id: Option<GameId>) -> Result<CommandList> {
        let result = server::handle_connect(&mut self.database, user_id, game_id)?;
        let to_update = match () {
            _ if user_id == self.user.id => &mut self.user,
            _ if user_id == self.opponent.id => &mut self.opponent,
            _ => panic!("Unknown user id: {:?}", user_id),
        };

        // Clear all previous state
        *to_update = TestClient::new(user_id);

        for command in result.commands.iter() {
            let c = command.command.as_ref().with_context(|| "command")?;
            to_update.handle_command(c);
        }

        Ok(result)
    }

    /// Execute a simulated client request for this game as a specific user,
    /// updating the client state as appropriate based on the responses.
    /// Returns the [GameResponse] for this action or an error if the server
    /// request failed.
    pub fn perform_action(&mut self, action: Action, player_id: PlayerId) -> Result<GameResponse> {
        let game_id = adapters::adapt_game_id(self.game_id());
        self.perform_action_with_game_id(action, player_id, Some(game_id))
    }

    /// Equivalent to [Self::perform_action] which allows the game id to be
    /// specified.
    pub fn perform_action_with_game_id(
        &mut self,
        action: Action,
        player_id: PlayerId,
        game_id: Option<GameIdentifier>,
    ) -> Result<GameResponse> {
        let response = server::handle_request(
            &mut self.database,
            &GameRequest {
                action: Some(GameAction { action: Some(action) }),
                game_id,
                player_id: Some(adapters::adapt_player_id(player_id)),
            },
        )?;

        let (opponent_id, local, remote) = self.opponent_local_remote(player_id);
        for command in &response.command_list.commands {
            local.handle_command(command.command.as_ref().expect("Empty command"));
        }

        if let Some((channel_user_id, list)) = &response.channel_response {
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
                room_id: adapters::adapt_room_id(room_id).into(),
            }),
            self.player_id_for_side(Side::Champion),
        )
        .expect("Server Error")
    }

    /// Adds a named card to its owner's hand.
    ///
    /// This function operates by locating a test card in the owner's deck and
    /// overwriting it with the provided `card_name`. This card is then
    /// moved to the user's hand via [GameState::move_card].
    /// CreateOrUpdateCard commands are sent to the attached test clients.
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
        self.database.game_mut().move_card(card_id, CardPosition::Hand(side));
        self.database.game_mut().card_mut(card_id).set_revealed_to(card_id.side, true);

        self.connect(self.user.id, Some(self.database.game().id)).expect("User connection error");
        self.connect(self.opponent.id, Some(self.database.game().id))
            .expect("Opponent connection error");

        adapters::adapt_card_id(card_id)
    }

    /// Creates and then plays a named card as the user who owns this card.
    ///
    /// This function first adds a copy of the requested card to the user's hand
    /// via [Self::add_to_hand]. The card is then played via the standard
    /// [PlayCardAction]. Action points and mana must be available and are spent
    /// as normal.
    ///
    /// If the card is a minion, project, or scheme card, it is played
    /// into the [crate::ROOM_ID] room. The [GameResponse] produced by
    /// playing the card is returned, along with its [CardIdentifier].
    ///
    /// Panics if the server returns an error for playing this card.
    pub fn play_from_hand(&mut self, card_name: CardName) -> CardIdentifier {
        self.play_in_room(card_name, crate::ROOM_ID)
    }

    /// Equivalent method to [Self::play_from_hand] which explicitly specifies
    /// the target room to use if this is a minion, project, scheme, or
    /// upgrade card.
    pub fn play_in_room(&mut self, card_name: CardName, room_id: RoomId) -> CardIdentifier {
        let card_id = self.add_to_hand(card_name);

        let target = match rules::get(card_name).card_type {
            CardType::Minion | CardType::Project | CardType::Scheme => Some(CardTarget {
                card_target: Some(card_target::CardTarget::RoomId(
                    adapters::adapt_room_id(room_id).into(),
                )),
            }),
            _ => None,
        };

        self.perform(
            Action::PlayCard(PlayCardAction { card_id: Some(card_id), target }),
            self.database.game().player(side_for_card_name(card_name)).id,
        );

        card_id
    }

    /// Locate a button containing the provided `text` in the provided player's
    /// main controls and invoke its registered action.
    pub fn click_on(&mut self, player_id: PlayerId, text: &'static str) -> GameResponse {
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

    pub fn player(&self, player_id: PlayerId) -> &TestClient {
        match () {
            _ if player_id == self.user.id => &self.user,
            _ if player_id == self.opponent.id => &self.opponent,
            _ => panic!("Unknown player id: {:?}", player_id),
        }
    }

    pub fn player_for_side(&self, side: Side) -> &TestClient {
        self.player(self.player_id_for_side(side))
    }

    pub fn player_id_for_side(&self, side: Side) -> PlayerId {
        if self.database.game().player(side).id == self.user.id {
            self.user.id
        } else if self.database.game().player(side).id == self.opponent.id {
            self.opponent.id
        } else {
            panic!("Cannot find PlayerId for side {:?}", side)
        }
    }

    /// Activates an ability of a card owned by the user
    pub fn activate_ability(&mut self, card_id: CardIdentifier, index: u32) {
        self.perform(
            Action::PlayCard(PlayCardAction {
                card_id: Some(CardIdentifier { ability_id: Some(index), ..card_id }),
                target: None,
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
    state.set_position(card.sorting_key, card.position());
    *game.card_mut(card_id) = state;
}

pub fn side_for_card_name(name: CardName) -> Side {
    rules::get(name).side
}

/// Represents a user client connected to a test game
#[derive(Clone)]
pub struct TestClient {
    pub id: PlayerId,
    pub data: ClientGameData,
    /// A player's view of *their own* state.
    pub this_player: ClientPlayer,
    /// A player's view of *their opponent's* state.
    pub other_player: ClientPlayer,
    pub interface: ClientInterface,
    pub cards: ClientCards,
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

        if let Command::RunInParallel(run_in_parallel) = command {
            for list in &run_in_parallel.commands {
                for command in &list.commands {
                    self.handle_command(command.command.as_ref().expect("Command"));
                }
            }
        }
    }
}

/// Simulated game state in an ongoing [TestSession]
#[derive(Clone, Default)]
pub struct ClientGameData {
    priority: Option<PlayerName>,
    raid_active: Option<bool>,
    object_positions: HashMap<GameObjectIdentifier, (u32, Position)>,
    last_message: Option<GameMessageType>,
}

impl ClientGameData {
    pub fn priority(&self) -> PlayerName {
        self.priority.unwrap()
    }

    pub fn raid_active(&self) -> bool {
        self.raid_active.expect("raid_active")
    }

    pub fn object_index_position(&self, id: Id) -> (u32, Position) {
        self.object_positions
            .get(&GameObjectIdentifier { id: Some(id) })
            .unwrap_or_else(|| panic!("No position available for {:?}", id))
            .clone()
    }

    pub fn object_position(&self, id: Id) -> Position {
        self.object_index_position(id).1
    }

    pub fn last_message(&self) -> GameMessageType {
        self.last_message.expect("Game Message")
    }

    fn update(&mut self, command: Command) {
        match command {
            Command::UpdateGameView(update_game) => {
                self.priority =
                    PlayerName::from_i32(update_game.game.as_ref().unwrap().current_priority);
                self.raid_active = Some(update_game.game.as_ref().unwrap().raid_active);
            }
            Command::MoveGameObjects(move_objects) => {
                for id in move_objects.ids {
                    let p = move_objects.position.as_ref().expect("ObjectPosition").clone();
                    self.object_positions
                        .insert(id, (p.sorting_key, p.position.expect("Position")));
                }
            }
            Command::DisplayGameMessage(display_message) => {
                self.last_message = GameMessageType::from_i32(display_message.message_type);
            }
            _ => {}
        }
    }
}

/// Simulated player state in an ongoing [TestSession]
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
                update.game.unwrap().user
            } else {
                update.game.unwrap().opponent
            });
        }
    }

    fn update_with_player(&mut self, player: Option<PlayerView>) {
        if let Some(p) = player {
            write_if_present(&mut self.mana, p.mana, |v| v.amount);
            write_if_present(&mut self.actions, p.action_tracker, |v| v.available_action_count);
            write_if_present(&mut self.score, p.score, |v| v.score);
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
        if let Command::RenderInterface(render) = command {
            if let Some(main_controls) = render.main_controls {
                self.main_controls = main_controls.node;
                self.card_anchors = main_controls.card_anchor_nodes;
            }
        }
    }
}

pub trait HasText {
    /// Returns true if there are any text nodes contained within this tree
    /// which contain the provided string.    
    fn has_text(&self, text: &'static str) -> bool;

    /// Populates `path` with the series of nodes leading to the node which
    /// contains the provided text. Leaves `path` unchanged if no node
    /// containing this text is found.
    fn find_text(&self, path: &mut Vec<Node>, text: &'static str);

    /// Finds the path to the provided `text` via [Self::find_text] and then
    /// searches up the path for a registered [EventHandlers].
    fn find_handlers(&self, text: &'static str) -> Option<EventHandlers>;
}

impl HasText for Node {
    fn has_text(&self, text: &'static str) -> bool {
        if let Some(NodeType { node_type: Some(node_type::NodeType::Text(s)) }) = &self.node_type {
            if s.label.contains(text) {
                return true;
            }
        }

        for child in &self.children {
            if child.has_text(text) {
                return true;
            }
        }

        false
    }

    fn find_text(&self, path: &mut Vec<Node>, text: &'static str) {
        if self.has_text(text) {
            path.push(self.clone());
        }

        for child in &self.children {
            child.find_text(path, text);
        }
    }

    fn find_handlers(&self, text: &'static str) -> Option<EventHandlers> {
        let mut nodes = vec![];
        self.find_text(&mut nodes, text);
        nodes.reverse();
        nodes.iter().find_map(|node| node.event_handlers.clone())
    }
}

impl HasText for Vec<&Node> {
    fn has_text(&self, text: &'static str) -> bool {
        for node in self {
            if node.has_text(text) {
                return true;
            }
        }
        false
    }

    fn find_text(&self, path: &mut Vec<Node>, text: &'static str) {
        for node in self {
            if node.has_text(text) {
                return node.find_text(path, text);
            }
        }
    }

    fn find_handlers(&self, text: &'static str) -> Option<EventHandlers> {
        for node in self {
            if let Some(handlers) = node.find_handlers(text) {
                return Some(handlers);
            }
        }
        None
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
        self.card_map.get(&card_id).expect("Card not found")
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

    /// Returns a player's discard pile in the same manner as [Self::hand]
    pub fn discard_pile(&self, player: PlayerName) -> Vec<String> {
        self.names_in_position(Position::DiscardPile(ObjectPositionDiscardPile {
            owner: player.into(),
        }))
    }

    /// Returns a vector containing the card titles in the provided `location`
    /// of a given room, Titles are structured in the same manner described
    /// in [Self::hand].
    pub fn room_cards(&self, room_id: RoomId, location: ClientRoomLocation) -> Vec<String> {
        self.names_in_position(Position::Room(ObjectPositionRoom {
            room_id: adapters::adapt_room_id(room_id).into(),
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
            Command::CreateOrUpdateCard(create_or_update) => {
                let card_view = create_or_update.clone().card.expect("CardView");
                self.card_map
                    .entry(card_view.card_id.expect("card_id"))
                    .and_modify(|c| c.update(&card_view))
                    .or_insert_with(|| ClientCard::new(&create_or_update));
            }
            Command::MoveGameObjects(move_objects) => {
                let position = move_objects.clone().position.expect("ObjectPosition");
                for id in move_objects.ids {
                    if let game_object_identifier::Id::CardId(identifier) = id.id.expect("ID") {
                        assert!(
                            self.card_map.contains_key(&identifier),
                            "Card not found (not created/already destroyed?) for {:?} -> {:?}",
                            identifier,
                            position
                        );
                        let mut card = self.card_map.get_mut(&identifier).unwrap();
                        card.position = Some(position.clone());
                    }
                }
            }
            Command::DestroyCard(destroy_card) => {
                self.card_map.remove(&destroy_card.card_id.expect("card_id"));
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
    arena_icon: Option<String>,
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

    pub fn arena_icon(&self) -> String {
        self.arena_icon.clone().expect("arena_icon")
    }

    fn new(command: &CreateOrUpdateCardCommand) -> Self {
        let mut result = Self { position: command.create_position.clone(), ..Self::default() };
        result.update(command.card.as_ref().expect("No CardView found"));
        result
    }

    fn update(&mut self, view: &CardView) {
        self.id = view.card_id;
        self.revealed_to_me = Some(view.revealed_to_viewer);
        self.is_face_up = Some(view.is_face_up);
        if let Some(revealed) = &view.revealed_card {
            self.update_revealed_card(revealed);
        }

        fn extract_arena_icon(view: &CardView) -> Option<&String> {
            view.card_icons.as_ref()?.arena_icon.as_ref()?.text.as_ref()
        }

        if let Some(icon) = extract_arena_icon(view) {
            self.arena_icon = Some(icon.clone());
        }
    }

    fn update_revealed_card(&mut self, revealed: &RevealedCardView) {
        self.can_play = Some(
            match revealed
                .targeting
                .as_ref()
                .expect("targeting")
                .targeting
                .as_ref()
                .expect("targeting")
            {
                Targeting::NoTargeting(NoTargeting { can_play }) => *can_play,
                Targeting::RoomTargeting(RoomTargeting { valid_rooms }) => valid_rooms.is_empty(),
            },
        );

        if let Some(title) = revealed.clone().title.map(|title| title.text) {
            self.title = Some(title);
        }
    }
}

impl PartialOrd for ClientCard {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.position.as_ref()?.sorting_key.partial_cmp(&other.position.as_ref()?.sorting_key)
    }
}

fn write_if_present<T, U>(value: &mut Option<T>, option: Option<U>, map: impl Fn(U) -> T) {
    if let Some(v) = option {
        *value = Some(map(v));
    }
}
