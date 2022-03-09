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

use data::card_name::CardName;
use data::deck::Deck;
use data::primitives::{GameId, PlayerId};
use display::adapters;
use insta::assert_snapshot;
use maplit::hashmap;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::{CreateNewGameAction, PlayerName, PlayerSide};
use test_utils::client::{HasText, TestSession};
use test_utils::fake_database::FakeDatabase;
use test_utils::summarize::Summary;
use test_utils::*;

#[test]
fn create_new_game() {
    let (game_id, overlord_id, champion_id) = generate_ids();
    let mut session = make_test_session(game_id, overlord_id, champion_id);
    let response = session.perform_action_with_game_id(
        Action::CreateNewGame(CreateNewGameAction {
            side: PlayerSide::Overlord.into(),
            opponent_id: Some(adapters::adapt_player_id(session.opponent_id())),
            deterministic: true,
        }),
        session.user_id(),
        None,
    );

    assert_snapshot!(Summary::run(&response));
}

#[test]
fn connect_to_new_game() {
    let (game_id, overlord_id, champion_id) = generate_ids();
    let mut session = make_test_session(game_id, overlord_id, champion_id);
    session
        .perform_action_with_game_id(
            Action::CreateNewGame(CreateNewGameAction {
                side: PlayerSide::Overlord.into(),
                opponent_id: Some(adapters::adapt_player_id(session.opponent_id())),
                deterministic: true,
            }),
            session.user_id(),
            None,
        )
        .expect("create game");
    let response = session.connect(overlord_id, Some(game_id));

    assert!(session.user.interface.controls().has_text("Keep"));
    assert!(session.user.interface.controls().has_text("Mulligan"));
    assert_eq!(5, session.user.cards.browser().len());
    assert_eq!(5, session.user.cards.hand(PlayerName::Opponent).len());

    assert_snapshot!(Summary::run(&response));
}

#[test]
fn keep_opening_hand() {
    let (game_id, overlord_id, champion_id) = generate_ids();
    let mut session = make_test_session(game_id, overlord_id, champion_id);
    session
        .perform_action_with_game_id(
            Action::CreateNewGame(CreateNewGameAction {
                side: PlayerSide::Overlord.into(),
                opponent_id: Some(adapters::adapt_player_id(session.opponent_id())),
                deterministic: true,
            }),
            session.user_id(),
            None,
        )
        .expect("create game");
    session.connect(overlord_id, Some(game_id)).expect("connect");
    session.connect(champion_id, Some(game_id)).expect("connect");

    let response = session.click_on(overlord_id, "Keep");
    assert!(session.user.interface.controls().has_text("Waiting"));
    assert_eq!(0, session.user.cards.browser().len());
    assert_eq!(5, session.user.cards.hand(PlayerName::User).len());
    assert_eq!(5, session.user.cards.hand(PlayerName::Opponent).len());

    assert_eq!(0, session.opponent.cards.hand(PlayerName::User).len());
    assert_eq!(5, session.opponent.cards.hand(PlayerName::Opponent).len());
    assert_eq!(5, session.opponent.cards.browser().len());

    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn mulligan_opening_hand() {
    let (game_id, overlord_id, champion_id) = generate_ids();
    let mut session = make_test_session(game_id, overlord_id, champion_id);
    session
        .perform_action_with_game_id(
            Action::CreateNewGame(CreateNewGameAction {
                side: PlayerSide::Overlord.into(),
                opponent_id: Some(adapters::adapt_player_id(session.opponent_id())),
                deterministic: true,
            }),
            session.user_id(),
            None,
        )
        .expect("create game");
    session.connect(overlord_id, Some(game_id)).expect("connect");
    session.connect(champion_id, Some(game_id)).expect("connect");

    let response = session.click_on(overlord_id, "Mulligan");
    assert_snapshot!(Summary::summarize(&response));

    assert!(session.user.interface.controls().has_text("Waiting"));
    assert_eq!(0, session.user.cards.browser().len());
    assert_eq!(5, session.user.cards.hand(PlayerName::User).len());
    assert_eq!(5, session.user.cards.hand(PlayerName::Opponent).len());

    assert_eq!(0, session.opponent.cards.hand(PlayerName::User).len());
    assert_eq!(5, session.opponent.cards.hand(PlayerName::Opponent).len());
    assert_eq!(5, session.opponent.cards.browser().len());
}

#[test]
fn both_keep_opening_hands() {
    let (game_id, overlord_id, champion_id) = generate_ids();
    let mut session = make_test_session(game_id, overlord_id, champion_id);
    session
        .perform_action_with_game_id(
            Action::CreateNewGame(CreateNewGameAction {
                side: PlayerSide::Overlord.into(),
                opponent_id: Some(adapters::adapt_player_id(session.opponent_id())),
                deterministic: true,
            }),
            session.user_id(),
            None,
        )
        .expect("create game");
    session.connect(overlord_id, Some(game_id)).expect("connect");
    session.connect(champion_id, Some(game_id)).expect("connect");

    session.click_on(overlord_id, "Keep");
    let response = session.click_on(champion_id, "Keep");
    assert_snapshot!(Summary::summarize(&response));

    assert_eq!(5, session.user.this_player.mana());
    assert_eq!(5, session.user.other_player.mana());
    assert_eq!(5, session.opponent.this_player.mana());
    assert_eq!(5, session.opponent.other_player.mana());

    assert_eq!(3, session.user.this_player.actions());
    assert_eq!(0, session.user.other_player.actions());
    assert_eq!(0, session.opponent.this_player.actions());
    assert_eq!(3, session.opponent.other_player.actions());

    assert!(session.dusk());
}

fn make_test_session(game_id: GameId, overlord_id: PlayerId, champion_id: PlayerId) -> TestSession {
    let database = FakeDatabase {
        generated_game_id: Some(game_id),
        game: None,
        overlord_deck: Some(Deck {
            owner_id: overlord_id,
            identity: CardName::TestOverlordIdentity,
            cards: hashmap! {CardName::TestOverlordSpell => 45},
        }),
        champion_deck: Some(Deck {
            owner_id: champion_id,
            identity: CardName::TestChampionIdentity,
            cards: hashmap! {CardName::TestChampionSpell => 45},
        }),
    };

    TestSession::new(database, overlord_id, champion_id)
}
