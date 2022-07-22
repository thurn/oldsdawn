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

//! Tools to facilitate testing. Should be included via wildcard import in all
//! tests.

#![allow(clippy::unwrap_in_result)]

pub mod client;
pub mod fake_database;
pub mod summarize;
pub mod test_games;

use std::collections::HashSet;
use std::fmt::Debug;
use std::fs;
use std::hash::Hash;
use std::sync::atomic::{AtomicU64, Ordering};

use adapters::ServerCardId;
use anyhow::Result;
use cards::initialize;
use data::card_name::CardName;
use data::card_state::{CardPosition, CardPositionKind};
use data::deck::Deck;
use data::game::{GameConfiguration, GamePhase, GameState, InternalRaidPhase, RaidData, TurnData};
use data::player_data::{CurrentGame, PlayerData};
use data::player_name::PlayerId;
use data::primitives::{
    ActionCount, CardId, Faction, GameId, ManaValue, PointsValue, RaidId, RoomId, Side,
};
use maplit::hashmap;
use prost::Message;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::{
    CardIdentifier, CommandList, GameCommand, LevelUpRoomAction, RoomIdentifier,
    SpendActionPointAction,
};
use rules::{dispatch, mana};

use crate::client::TestSession;
use crate::fake_database::FakeDatabase;

pub static NEXT_ID: AtomicU64 = AtomicU64::new(1_000_000);
/// The title returned for hidden cards
pub const HIDDEN_CARD: &str = "Hidden Card";
/// [RoomId] used by default for targeting
pub const ROOM_ID: RoomId = RoomId::RoomA;
/// Client equivalent of [ROOM_ID].
pub const CLIENT_ROOM_ID: RoomIdentifier = RoomIdentifier::RoomA;
/// Default Raid ID to use during testing
pub const RAID_ID: RaidId = RaidId(1);
/// Default mana for players in a test game if not otherwise specified
pub const STARTING_MANA: ManaValue = 999;

/// Creates a new game with the user playing as the `user_side` player.
///
/// By default, this creates a new game with both player's decks populated with
/// blank test cards and all other game zones empty (no cards are drawn). The
/// game is advanced to the user's first turn. See [Args] for information about
/// the default configuration options and how to modify them.
pub fn new_game(user_side: Side, args: Args) -> TestSession {
    initialize::run();
    let (game_id, user_id, opponent_id) = generate_ids();
    let (overlord_user, champion_user) = match user_side {
        Side::Overlord => (user_id, opponent_id),
        Side::Champion => (opponent_id, user_id),
    };

    let overlord_deck = Deck {
        owner_id: overlord_user,
        side: Side::Overlord,
        identity: CardName::TestOverlordIdentity,
        cards: hashmap! {CardName::TestOverlordSpell => 45},
    };
    let champion_deck = Deck {
        owner_id: champion_user,
        side: Side::Champion,
        identity: CardName::TestChampionIdentity,
        cards: hashmap! {CardName::TestChampionSpell => 45},
    };

    let mut game = GameState::new(
        game_id,
        overlord_deck,
        champion_deck,
        GameConfiguration { deterministic: true, ..GameConfiguration::default() },
    );
    dispatch::populate_delegate_cache(&mut game);

    let turn_side = args.turn.unwrap_or(user_side);
    game.data.phase = GamePhase::Play;
    game.data.turn = TurnData { side: turn_side, turn_number: 0 };
    mana::set(&mut game, user_side, args.mana);
    game.player_mut(user_side).score = args.score;
    mana::set(&mut game, user_side.opponent(), args.opponent_mana);
    game.player_mut(user_side.opponent()).score = args.opponent_score;
    game.player_mut(turn_side).actions = args.actions;

    set_deck_top(&mut game, user_side, args.deck_top);
    set_deck_top(&mut game, user_side.opponent(), args.opponent_deck_top);
    set_discard_pile(&mut game, user_side, args.discard);
    set_discard_pile(&mut game, user_side.opponent(), args.opponent_discard);

    if args.add_raid {
        game.data.raid = Some(RaidData {
            raid_id: RAID_ID,
            target: ROOM_ID,
            internal_phase: InternalRaidPhase::Begin,
            encounter: None,
            accessed: vec![],
            jump_request: None,
        })
    }

    let database = FakeDatabase {
        generated_game_id: None,
        game: Some(game),
        players: hashmap! {
            overlord_user => PlayerData {
                id: overlord_user,
                current_game: Some(CurrentGame::Playing(game_id)),
                decks: vec![],
                collection: hashmap! {}
            },
            champion_user => PlayerData {
                id: champion_user,
                current_game: Some(CurrentGame::Playing(game_id)),
                decks: vec![],
                collection: hashmap! {}
            }
        },
    };

    let mut session = TestSession::new(database, user_id, opponent_id);
    let (user_hand_card, opponent_hand_card) = if user_side == Side::Overlord {
        (CardName::TestOverlordSpell, CardName::TestChampionSpell)
    } else {
        (CardName::TestChampionSpell, CardName::TestOverlordSpell)
    };

    for _ in 0..args.hand_size {
        session.add_to_hand(user_hand_card);
    }
    for _ in 0..args.opponent_hand_size {
        session.add_to_hand(opponent_hand_card);
    }

    if args.connect {
        session.connect(user_id).expect("Connection failed");
        session.connect(opponent_id).expect("Connection failed");
    }
    session
}

pub fn generate_ids() -> (GameId, PlayerId, PlayerId) {
    let next_id = NEXT_ID.fetch_add(2, Ordering::SeqCst);
    (GameId::new(next_id), PlayerId::Database(next_id), PlayerId::Database(next_id + 1))
}

/// Arguments to [new_game]
#[derive(Clone, Debug)]
pub struct Args {
    /// Player whose turn it should be. Defaults to the `user_side` player.
    pub turn: Option<Side>,
    /// Mana available for the `user_side` player. Defaults to 999
    /// ([STARTING_MANA]).
    pub mana: ManaValue,
    /// Mana for the opponent of the `user_side` player. Defaults to 999
    /// ([STARTING_MANA]).
    pub opponent_mana: ManaValue,
    /// Actions available for the `turn` player. Defaults to 3.
    pub actions: ActionCount,
    /// Score for the `user_side` player. Defaults to 0.
    pub score: PointsValue,
    /// Score for the opponent of the `user_side` player. Defaults to 0.
    pub opponent_score: PointsValue,
    /// Starting size for the `user_side` player's hand, draw from the top of
    /// their deck. Hand will consist entirely of 'test spell' cards.
    /// Defaults to 0.
    pub hand_size: u64,
    /// Starting size for the opponent player's hand, draw from the top of their
    /// deck. Hand will consist entirely of 'test spell' cards. Defaults to
    /// 0.
    pub opponent_hand_size: u64,
    /// Card to be inserted into the `user_side` player's deck as the next draw.
    ///
    /// This card will be drawn when drawing randomly from the deck (as long as
    /// no known cards are placed on top of it) because the game is created with
    /// [GameConfiguration::deterministic] set to true.
    pub deck_top: Option<CardName>,
    /// Card to be inserted into the opponent player's deck as the next draw.
    pub opponent_deck_top: Option<CardName>,
    /// Card to be inserted into the `user_side` player's discard pile.
    pub discard: Option<CardName>,
    /// Card to be inserted into the opponent player's discard pile.
    pub opponent_discard: Option<CardName>,
    /// Set up an active raid within the created game using [ROOM_ID] as the
    /// target and [RAID_ID] as the ID.    
    pub add_raid: bool,
    /// If false, will not attempt to automatically connect to this game.
    /// Defaults to true.
    pub connect: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            turn: None,
            mana: STARTING_MANA,
            opponent_mana: STARTING_MANA,
            actions: 3,
            score: 0,
            opponent_score: 0,
            hand_size: 0,
            opponent_hand_size: 0,
            deck_top: None,
            opponent_deck_top: None,
            discard: None,
            opponent_discard: None,
            add_raid: false,
            connect: true,
        }
    }
}

fn set_deck_top(game: &mut GameState, side: Side, deck_top: Option<CardName>) {
    if let Some(deck_top) = deck_top {
        let target_id = game
            .cards(side)
            .iter()
            .find(|c| c.position().kind() == CardPositionKind::DeckUnknown)
            .expect("No cards in deck")
            .id;
        client::overwrite_card(game, target_id, deck_top);
        game.move_card_internal(target_id, CardPosition::DeckTop(side))
    }
}

fn set_discard_pile(game: &mut GameState, side: Side, discard: Option<CardName>) {
    if let Some(discard) = discard {
        let target_id = game
            .cards(side)
            .iter()
            .filter(|c| c.position().kind() == CardPositionKind::DeckUnknown)
            .last() // Take last to avoid overwriting deck top
            .expect("No cards in deck")
            .id;
        client::overwrite_card(game, target_id, discard);
        game.move_card_internal(target_id, CardPosition::DiscardPile(side));
        game.card_mut(target_id).turn_face_down();
    }
}

pub fn spend_actions_until_turn_over(session: &mut TestSession, side: Side) {
    let id = session.player_id_for_side(side);
    while session.player(id).this_player.actions() > 0 {
        session.perform(Action::SpendActionPoint(SpendActionPointAction {}), id);
    }
}

/// Levels up the [CLIENT_ROOM_ID] room a specified number of `times`. If this
/// requires multiple turns, spends the Champion turns doing nothing.
///
/// NOTE that this may cause the Champion to draw cards for their turn.
pub fn level_up_room(session: &mut TestSession, times: u32) {
    let mut levels = 0;
    let overlord_id = session.player_id_for_side(Side::Overlord);

    loop {
        while session.player(overlord_id).this_player.actions() > 0 {
            session.perform(
                Action::LevelUpRoom(LevelUpRoomAction { room_id: CLIENT_ROOM_ID.into() }),
                overlord_id,
            );
            levels += 1;

            if levels == times {
                return;
            }
        }

        assert!(session.dawn());
        spend_actions_until_turn_over(session, Side::Champion);
        assert!(session.dusk());
    }
}

/// Must be invoked during the Overlord turn. Performs the following actions:
/// - Plays a test Scheme card
///  - Ends the Overlord turn
///  - Initiates a raid on the [ROOM_ID] room
///
/// NOTE: This causes the Champion player to draw a card for their turn!
pub fn set_up_minion_combat(session: &mut TestSession) {
    set_up_minion_combat_with_action(session, |_| {});
}

/// Equivalent to [set_up_minion_combat] which invokes an `action` function at
/// the start of the Champion's turn.
pub fn set_up_minion_combat_with_action(
    session: &mut TestSession,
    action: impl FnOnce(&mut TestSession),
) {
    session.play_from_hand(CardName::TestScheme31);
    spend_actions_until_turn_over(session, Side::Overlord);
    assert!(session.dawn());
    action(session);
    session.initiate_raid(ROOM_ID);
}

pub fn minion_for_faction(faction: Faction) -> CardName {
    match faction {
        Faction::Mortal => CardName::TestMortalMinion,
        Faction::Abyssal => CardName::TestAbyssalMinion,
        Faction::Infernal => CardName::TestInfernalMinion,
        _ => panic!("Unsupported"),
    }
}

/// Must be invoked during the Champion turn. Performs the following actions:
///
/// - Ends the Champion turn
/// - Plays a 3-1 scheme in the [ROOM_ID] room.
/// - Plays the provided `card_name` minion into that room.
/// - Plays the selected minion in the [ROOM_ID] room.
/// - Ends the Overlord turn.
///
/// Returns a tuple of (scheme_id, minion_id).
///
/// WARNING: This causes both players to draw cards for their turns!
pub fn setup_raid_target(
    session: &mut TestSession,
    card_name: CardName,
) -> (CardIdentifier, CardIdentifier) {
    spend_actions_until_turn_over(session, Side::Champion);
    assert!(session.dusk());
    let scheme_id = session.play_from_hand(CardName::TestScheme31);
    let minion_id = session.play_from_hand(card_name);
    spend_actions_until_turn_over(session, Side::Overlord);
    assert!(session.dawn());
    (scheme_id, minion_id)
}

pub fn click_on_continue(session: &mut TestSession) {
    session.click_on(session.player_id_for_side(Side::Champion), "Continue");
}

pub fn click_on_score(session: &mut TestSession) {
    session.click_on(session.player_id_for_side(Side::Champion), "Score");
}

pub fn click_on_end_raid(session: &mut TestSession) {
    session.click_on(session.player_id_for_side(Side::Champion), "End Raid");
}

/// Must be invoked during the Champion turn. Performs the following actions:
///
/// - Performs all actions described in [setup_raid_target].
/// - Initiates a raid on the [ROOM_ID] room.
/// - Activates the room
/// - Clicks on the button with text matching `name` in order to fire weapon
///   abilities.
///
/// WARNING: This causes both players to draw cards for their turns!
pub fn fire_weapon_combat_abilities(
    session: &mut TestSession,
    faction: Faction,
    name: &'static str,
) {
    setup_raid_target(session, minion_for_faction(faction));
    session.initiate_raid(ROOM_ID);
    session.click_on(session.player_id_for_side(Side::Champion), name);
}

/// Asserts that the display names of the provided vector of [CardName]s are
/// precisely identical to the provided vector of strings.
pub fn assert_identical(expected: Vec<CardName>, actual: Vec<String>) {
    let set = expected.iter().map(CardName::displayed_name).collect::<Vec<_>>();
    assert_eq!(set, actual);
}

/// Asserts two vectors contain the same elements in any order
pub fn assert_contents_equal<T: Eq + Hash + Debug>(left: Vec<T>, right: Vec<T>) {
    let left_count = left.len();
    let right_count = right.len();
    let left_set: HashSet<T> = left.into_iter().collect();
    let right_set: HashSet<T> = right.into_iter().collect();
    assert_eq!(left_set.len(), left_count);
    assert_eq!(right_set.len(), right_count);
    assert_eq!(left_set, right_set);
}

/// Asserts that a [Result] is not an error
pub fn assert_ok<T: Debug, E: Debug>(result: &Result<T, E>) {
    assert!(result.is_ok(), "Unexpected error, got {:?}", result)
}

/// Asserts that a [Result] is an error
pub fn assert_error<T: Debug, E: Debug>(result: Result<T, E>) {
    assert!(result.is_err(), "Expected an error, got {:?}", result)
}

/// Creates a [CardIdentifier] representing the ability with the provided
/// `index` of this `card_id`.
pub fn ability_id(card_id: CardIdentifier, ability: u32) -> CardIdentifier {
    CardIdentifier { ability_id: Some(ability), ..card_id }
}

/// Converts a [CardIdentifier] into a [CardId].
pub fn server_card_id(card_id: CardIdentifier) -> CardId {
    match adapters::server_card_id(card_id) {
        Ok(ServerCardId::CardId(id)) => id,
        _ => panic!("Expected server card id"),
    }
}

pub fn create_test_recording(session: &TestSession, name: &str) {
    record_output_for_side(session, format!("{}_overlord", name), Side::Overlord).unwrap();
    record_output_for_side(session, format!("{}_champion", name), Side::Champion).unwrap();
}

pub fn record_output_for_side(session: &TestSession, name: String, side: Side) -> Result<()> {
    let commands = CommandList {
        commands: session
            .player_for_side(side)
            .history
            .iter()
            .map(|c| GameCommand { command: Some(c.clone()) })
            .collect(),
    };
    let encoded = commands.encode_length_delimited_to_vec();
    fs::write(format!("../../Assets/Resources/TestRecordings/test_{}.bytes", name), encoded)?;

    Ok(())
}
