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
pub mod summarize;
pub mod test_games;

use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Result;
use data::card_name::CardName;
use data::card_state::CardPositionKind;
use data::deck::Deck;
use data::game::{GameConfiguration, GameState, RaidData, RaidPhase};
use data::primitives::{
    ActionCount, GameId, ManaValue, PlayerId, PointsValue, RaidId, RoomId, Side,
};
use maplit::hashmap;
use protos::spelldawn::RoomIdentifier;
use server::GameResponse;

use crate::client::TestGame;
pub static NEXT_ID: AtomicU64 = AtomicU64::new(1_000_000);
/// The title returned for hidden cards
pub const HIDDEN_CARD: &str = "Hidden Card";
/// [RoomId] used by default for targeting
pub const ROOM_ID: RoomId = RoomId::RoomA;
/// Client equivalent of [ROOM_ID].
pub const CLIENT_ROOM_ID: RoomIdentifier = RoomIdentifier::RoomA;
/// Default Raid ID to use during testing
pub const RAID_ID: RaidId = RaidId(1);

/// Creates a new game with the user playing as the `user_side` player.
///
/// By default, this creates a new game with both player's decks populated with
/// blank test cards and all other game zones empty (no cards are drawn). The
/// game is advanced to the user's first turn. See [Args] for information about
/// the default configuration options and how to modify them.
pub fn new_game(user_side: Side, args: Args) -> TestGame {
    let discovered = cards::initialize();
    println!("Discovered {} cards", discovered);
    let (game_id, user_id, opponent_id) = generate_ids();
    let (overlord_user, champion_user) = match user_side {
        Side::Overlord => (user_id, opponent_id),
        Side::Champion => (opponent_id, user_id),
    };

    let mut game = GameState::new_game(
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

    game.data.turn = args.turn.unwrap_or(user_side);
    game.player_mut(user_side).mana = args.mana;
    game.player_mut(user_side).score = args.score;
    game.player_mut(user_side.opponent()).mana = args.opponent_mana;
    game.player_mut(user_side.opponent()).score = args.opponent_score;
    game.player_mut(game.data.turn).actions = args.turn_actions;

    set_deck_top(&mut game, user_side, args.deck_top);
    set_deck_top(&mut game, user_side.opponent(), args.opponent_deck_top);

    if let Some(raid) = args.raid {
        game.data.raid = Some(RaidData {
            raid_id: RAID_ID,
            target: ROOM_ID,
            phase: raid.phase,
            active: false,
            accessed: vec![],
        })
    }

    let mut game = TestGame::new(game, user_id, opponent_id);
    if args.connect {
        game.connect(user_id, Some(game_id)).expect("Connection failed");
        game.connect(opponent_id, Some(game_id)).expect("Connection failed");
    }
    game
}

fn generate_ids() -> (GameId, PlayerId, PlayerId) {
    let next_id = NEXT_ID.fetch_add(2, Ordering::SeqCst);
    (GameId::new(next_id), PlayerId::new(next_id), PlayerId::new(next_id + 1))
}

/// Arguments to [new_game]
#[derive(Clone, Debug)]
pub struct Args {
    /// Player whose turn it should be. Defaults to the `user_side` player.
    pub turn: Option<Side>,
    /// Mana available for the `user_side` player. Defaults to 999.
    pub mana: ManaValue,
    /// Mana for the opponent of the `user_side` player. Defaults to 999.
    pub opponent_mana: ManaValue,
    /// Actions available for the `turn` player. Defaults to 3.
    pub turn_actions: ActionCount,
    /// Score for the `user_side` player. Defaults to 0.
    pub score: PointsValue,
    /// Score for the opponent of the `user_side` player. Defaults to 0.
    pub opponent_score: PointsValue,
    /// Card to be inserted into the `user_side` player's deck as the next draw.
    ///
    /// This card will be drawn when drawing randomly from the deck (as long as
    /// no known cards are placed on top of it) because the game is created with
    /// [GameConfiguration::deterministic] set to true.
    pub deck_top: Option<CardName>,
    /// Card to be inserted into the opponent player's deck as the next draw.
    pub opponent_deck_top: Option<CardName>,
    /// Set up an active raid within the created game using [ROOM_ID] as the
    /// target and [RAID_ID] as the ID.
    pub raid: Option<TestRaid>,
    /// If false, will not attempt to automatically connect to this game.
    /// Defaults to true.
    pub connect: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            turn: None,
            mana: 999,
            opponent_mana: 999,
            turn_actions: 3,
            score: 0,
            opponent_score: 0,
            deck_top: None,
            opponent_deck_top: None,
            raid: None,
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

/// Helper function to invoke [assert_commands_match_lists] with the same
/// command names for both players.
pub fn assert_commands_match(response: &Result<GameResponse>, names: Vec<&str>) {
    assert_commands_match_lists(response, names.clone(), names.clone());
}

/// Asserts that each player receives the named commands
pub fn assert_commands_match_lists(
    response: &Result<GameResponse>,
    local_names: Vec<&str>,
    remote_names: Vec<&str>,
) {
    let value = response.as_ref().expect("Server error");
    let local = value.command_list.commands.iter().map(server::command_name).collect::<Vec<_>>();
    assert_eq!(local_names, local, "Local commands do not match expected");
    let remote = value
        .channel_response
        .clone()
        .expect("Channel Response")
        .1
        .commands
        .iter()
        .map(server::command_name)
        .collect::<Vec<_>>();
    assert_eq!(remote_names, remote, "Remote commands do not match expected");
}
