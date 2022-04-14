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

use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Result;
use data::card_name::CardName;
use data::card_state::{CardPosition, CardPositionKind};
use data::deck::Deck;
use data::game::{CurrentTurn, GameConfiguration, GamePhase, GameState, RaidData, RaidPhase};
use data::primitives::{
    ActionCount, GameId, ManaValue, PlayerId, PointsValue, RaidId, RoomId, Side,
};
use maplit::hashmap;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::{DrawCardAction, GainManaAction, LevelUpRoomAction, RoomIdentifier};

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
    let discovered = cards::initialize();
    println!("Discovered {} cards", discovered);
    let (game_id, user_id, opponent_id) = generate_ids();
    let (overlord_user, champion_user) = match user_side {
        Side::Overlord => (user_id, opponent_id),
        Side::Champion => (opponent_id, user_id),
    };

    let mut game = GameState::new(
        game_id,
        Deck {
            owner_id: overlord_user,
            identity: CardName::TestOverlordIdentity,
            cards: hashmap! {CardName::TestOverlordSpell => 45},
        },
        Deck {
            owner_id: champion_user,
            identity: CardName::TestChampionIdentity,
            cards: hashmap! {CardName::TestChampionSpell => 45},
        },
        GameConfiguration { deterministic: true, ..GameConfiguration::default() },
    );

    if !args.resolve_mulligans {
        let turn_side = args.turn.unwrap_or(user_side);
        game.data.phase = GamePhase::Play(CurrentTurn { side: turn_side, turn_number: 0 });
        game.player_mut(user_side).mana = args.mana;
        game.player_mut(user_side).score = args.score;
        game.player_mut(user_side.opponent()).mana = args.opponent_mana;
        game.player_mut(user_side.opponent()).score = args.opponent_score;
        game.player_mut(turn_side).actions = args.actions;

        set_deck_top(&mut game, user_side, args.deck_top);
        set_deck_top(&mut game, user_side.opponent(), args.opponent_deck_top);
        set_discard_pile(&mut game, user_side, args.discard);
        set_discard_pile(&mut game, user_side.opponent(), args.opponent_discard);

        if let Some(raid) = args.raid {
            game.data.raid = Some(RaidData {
                raid_id: RAID_ID,
                target: ROOM_ID,
                phase: raid.phase,
                room_active: false,
                accessed: vec![],
            })
        }
    }

    let database = FakeDatabase {
        generated_game_id: None,
        game: Some(game),
        overlord_deck: None,
        champion_deck: None,
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
        session.connect(user_id, Some(game_id)).expect("Connection failed");
        session.connect(opponent_id, Some(game_id)).expect("Connection failed");
    }
    session
}

pub fn generate_ids() -> (GameId, PlayerId, PlayerId) {
    let next_id = NEXT_ID.fetch_add(2, Ordering::SeqCst);
    (GameId::new(next_id), PlayerId::new(next_id), PlayerId::new(next_id + 1))
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
    pub raid: Option<TestRaid>,
    /// If true, will create the game in the "resolve mulligans" phase instead
    /// of automatically advancing to the user's first turn. Defaults to
    /// false.
    ///
    /// If specified all game state configuration options are silently ignored.
    pub resolve_mulligans: bool,
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
            raid: None,
            resolve_mulligans: false,
            connect: true,
        }
    }
}

/// Options for a test raid
#[derive(Clone, Debug)]
pub struct TestRaid {
    pub phase: RaidPhase,
    pub active: bool,
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
        game.move_card(target_id, CardPosition::DiscardPile(side));
        game.card_mut(target_id).turn_face_down();
    }
}

pub fn gain_mana_until_turn_over(session: &mut TestSession, side: Side) -> u32 {
    let id = session.player_id_for_side(side);
    let mut result = 0;
    while session.player(id).this_player.actions() > 0 {
        session.perform(Action::GainMana(GainManaAction {}), id);
        result += 1;
    }
    result
}

pub fn draw_cards_until_turn_over(session: &mut TestSession, side: Side) {
    let id = session.player_id_for_side(side);
    while session.player(id).this_player.actions() > 0 {
        session.perform(Action::DrawCard(DrawCardAction {}), id);
    }
}

/// Levels up the [CLIENT_ROOM_ID] room a specified number of `times`. If this
/// requires multiple turns, spends the Champion turns gaining mana.
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
        gain_mana_until_turn_over(session, Side::Champion);
        assert!(session.dusk());
    }
}

/// Ends the Overlord turn, initiates a raid on the [ROOM_ID] room, activates
/// the room, and then continues to trigger a minion combat ability.
///
/// NOTE: This causes the Champion player to draw a card for their turn!
pub fn end_turn_initiate_raid_fire_combat_abilities(session: &mut TestSession) {
    gain_mana_until_turn_over(session, Side::Overlord);
    assert!(session.dawn());
    session.initiate_raid(ROOM_ID);
    session.click_on(session.player_id_for_side(Side::Overlord), "Activate");
    session.click_on(session.player_id_for_side(Side::Champion), "Continue");
}

/// Asserts that the display names of the provided vector of [CardName]s are
/// precisely identical to the provided vector of strings.
pub fn assert_identical(expected: Vec<CardName>, actual: Vec<String>) {
    let set = expected.iter().map(CardName::displayed_name).collect::<Vec<_>>();
    assert_eq!(set, actual);
}

/// Asserts that a [Result] is not an error
pub fn assert_ok<T: Debug, E: Debug>(result: &Result<T, E>) {
    assert!(result.is_ok(), "Unexpected error, got {:?}", result)
}

/// Asserts that a [Result] is an error
pub fn assert_error<T: Debug, E: Debug>(result: Result<T, E>) {
    assert!(result.is_err(), "Expected an error, got {:?}", result)
}
