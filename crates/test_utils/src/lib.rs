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

pub mod client;

use std::fmt::Debug;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Result;
use data::card_name::CardName;
use data::card_state::CardPositionKind;
use data::deck::Deck;
use data::game::{GameConfiguration, GameState, RaidState};
use data::primitives::{ActionCount, GameId, ManaValue, PointsValue, RaidId, RoomId, Side, UserId};
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
    let (game_id, user_id, opponent_id) = generate_ids(args.id_basis);
    let (overlord_user, champion_user) = match user_side {
        Side::Overlord => (user_id, opponent_id),
        Side::Champion => (opponent_id, user_id),
    };

    let mut state = GameState::new_game(
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

    state.data.turn = user_side;
    state.player_mut(user_side).mana = args.mana;
    state.player_mut(user_side).actions = args.actions;
    state.player_mut(user_side).score = args.score;

    if let Some(next_draw) = args.next_draw {
        let target_id = state
            .cards(user_side)
            .iter()
            .find(|c| c.position.kind() == CardPositionKind::DeckUnknown)
            .expect("No cards in deck")
            .id;
        client::overwrite_card(&mut state, target_id, next_draw);
    }

    if let Some(raid) = args.raid {
        state.data.raid =
            Some(RaidState { raid_id: RAID_ID, encounter_number: 0, priority: raid.priority })
    }

    let mut game = TestGame::new(state, user_id, opponent_id);
    if args.connect {
        game.connect(user_id, Some(game_id)).expect("Connection failed");
        game.connect(opponent_id, Some(game_id)).expect("Connection failed");
    }
    game
}

fn generate_ids(basis: Option<u64>) -> (GameId, UserId, UserId) {
    let next_id = basis.unwrap_or_else(|| NEXT_ID.fetch_add(2, Ordering::SeqCst));
    (GameId::new(next_id), UserId::new(next_id), UserId::new(next_id + 1))
}

/// Arguments to [new_game]
#[derive(Clone, Debug)]
pub struct Args {
    /// Value to use for generated GameID and UserIds, in order to ensure
    /// deterministic snapshots. Game ID and User ID will use this number,
    /// Opponent ID will use this number + 1. Must be less than 1,000,000.
    pub id_basis: Option<u64>,
    /// Mana available for the `user_side` player. Defaults to 5.
    pub mana: ManaValue,
    /// Actions available for the `user_side` player. Defaults to 3.
    pub actions: ActionCount,
    /// Score for the `user_side` player. Defaults to 0.
    pub score: PointsValue,
    /// Card to be inserted into the `user_side` player's deck as the next draw.
    ///
    /// This card will be drawn when drawing randomly from the deck (as long as
    /// no known cards are placed on top of it) because the game is created with
    /// [GameConfiguration::deterministic] set to true.
    pub next_draw: Option<CardName>,
    /// Set up an active raid within the created game
    pub raid: Option<TestRaid>,
    /// If false, will not attempt to automatically connect to this game.
    /// Defaults to true.
    pub connect: bool,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            id_basis: None,
            mana: 5,
            actions: 3,
            score: 0,
            next_draw: None,
            raid: None,
            connect: true,
        }
    }
}

/// Options for an active test raid
#[derive(Clone, Debug)]
pub struct TestRaid {
    pub priority: Side,
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
