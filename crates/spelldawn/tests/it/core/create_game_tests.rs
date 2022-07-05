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

use cards::initialize;
use data::card_name::CardName;
use data::deck::Deck;
use data::player_data::PlayerData;
use data::player_name::PlayerId;
use data::primitives::{GameId, Side};
use insta::assert_snapshot;
use maplit::hashmap;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{
    DeckIdentifier, NewGameAction, NewGameDebugOptions, PlayerIdentifier, PlayerName,
};
use test_utils::client::{HasText, TestSession};
use test_utils::fake_database::FakeDatabase;
use test_utils::summarize::Summary;
use test_utils::*;

static OVERLORD_DECK_ID: DeckIdentifier = DeckIdentifier { value: 0 };
static CHAMPION_DECK_ID: DeckIdentifier = DeckIdentifier { value: 1 };

#[test]
fn create_new_game() {
    let (game_id, overlord_id, champion_id) = generate_ids();
    let mut session = make_overlord_test_session(game_id, overlord_id, champion_id);
    let response = session.perform_action(
        Action::NewGame(NewGameAction {
            deck: Some(OVERLORD_DECK_ID),
            opponent_id: opponent_id(&session),
            debug_options: Some(NewGameDebugOptions {
                deterministic: true,
                ..NewGameDebugOptions::default()
            }),
        }),
        session.user_id(),
    );
    assert_eq!(session.connect(overlord_id).unwrap().commands.len(), 0);
    assert_snapshot!(Summary::run(&response));
}

#[test]
fn connect_to_new_game() {
    let (game_id, overlord_id, champion_id) = generate_ids();
    let mut session = make_overlord_test_session(game_id, overlord_id, champion_id);
    initiate_game(&mut session);

    let response = session.connect(overlord_id);
    assert!(session.user.interface.controls().has_text("Keep"));
    assert!(session.user.interface.controls().has_text("Mulligan"));
    assert_eq!(5, session.user.cards.revealed_cards().len());
    assert_eq!(5, session.user.cards.hand(PlayerName::Opponent).len());

    assert_snapshot!(Summary::run(&response));
}

#[test]
fn keep_opening_hand() {
    let (game_id, overlord_id, champion_id) = generate_ids();
    let mut session = make_overlord_test_session(game_id, overlord_id, champion_id);
    initiate_game(&mut session);

    let response = session.click_on(overlord_id, "Keep");
    assert!(session.user.interface.controls().has_text("Waiting"));
    assert_eq!(0, session.user.cards.revealed_cards().len());
    assert_eq!(5, session.user.cards.hand(PlayerName::User).len());
    assert_eq!(5, session.user.cards.hand(PlayerName::Opponent).len());

    assert_eq!(0, session.opponent.cards.hand(PlayerName::User).len());
    assert_eq!(5, session.opponent.cards.hand(PlayerName::Opponent).len());
    assert_eq!(5, session.opponent.cards.revealed_cards().len());

    assert_snapshot!(Summary::summarize(&response));
}

#[test]
fn mulligan_opening_hand() {
    let (game_id, overlord_id, champion_id) = generate_ids();
    let mut session = make_overlord_test_session(game_id, overlord_id, champion_id);
    initiate_game(&mut session);

    let response = session.click_on(overlord_id, "Mulligan");
    assert_snapshot!(Summary::summarize(&response));

    assert!(session.user.interface.controls().has_text("Waiting"));
    assert_eq!(0, session.user.cards.revealed_cards().len());
    assert_eq!(5, session.user.cards.hand(PlayerName::User).len());
    assert_eq!(5, session.user.cards.hand(PlayerName::Opponent).len());

    assert_eq!(0, session.opponent.cards.hand(PlayerName::User).len());
    assert_eq!(5, session.opponent.cards.hand(PlayerName::Opponent).len());
    assert_eq!(5, session.opponent.cards.revealed_cards().len());
}

#[test]
fn both_keep_opening_hands() {
    let (game_id, overlord_id, champion_id) = generate_ids();
    let mut session = make_overlord_test_session(game_id, overlord_id, champion_id);
    initiate_game(&mut session);

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

fn user_id(session: &TestSession) -> Option<PlayerIdentifier> {
    Some(fake_database::to_player_identifier(session.user_id()))
}

fn opponent_id(session: &TestSession) -> Option<PlayerIdentifier> {
    Some(fake_database::to_player_identifier(session.opponent_id()))
}

/// Creates a [TestSession] for the Overlord player. Both players have their
/// decks populated, but neither has submitted a 'new game' request.
fn make_overlord_test_session(
    game_id: GameId,
    overlord_id: PlayerId,
    champion_id: PlayerId,
) -> TestSession {
    initialize::run();
    let overlord_deck = Deck {
        owner_id: overlord_id,
        side: Side::Overlord,
        identity: CardName::TestOverlordIdentity,
        cards: hashmap! {CardName::TestOverlordSpell => 45},
    };
    let champion_deck = Deck {
        owner_id: champion_id,
        side: Side::Champion,
        identity: CardName::TestChampionIdentity,
        cards: hashmap! {CardName::TestChampionSpell => 45},
    };

    let database = FakeDatabase {
        generated_game_id: Some(game_id),
        game: None,
        players: hashmap! {
            overlord_id => PlayerData {
                id: overlord_id,
                current_game: None,
                decks: vec![overlord_deck.clone(), champion_deck.clone()],
                collection: hashmap! {}
            },
            champion_id => PlayerData {
                id: champion_id,
                current_game: None,
                decks: vec![overlord_deck, champion_deck],
                collection: hashmap! {}
            }
        },
    };

    TestSession::new(database, overlord_id, champion_id)
}

fn initiate_game(session: &mut TestSession) {
    session.perform(
        Action::NewGame(NewGameAction {
            deck: Some(CHAMPION_DECK_ID),
            opponent_id: user_id(session),
            debug_options: Some(NewGameDebugOptions {
                deterministic: true,
                ..NewGameDebugOptions::default()
            }),
        }),
        session.opponent_id(),
    );
    let action2 = session
        .perform_action(
            Action::NewGame(NewGameAction {
                deck: Some(OVERLORD_DECK_ID),
                opponent_id: opponent_id(session),
                debug_options: Some(NewGameDebugOptions {
                    deterministic: true,
                    ..NewGameDebugOptions::default()
                }),
            }),
            session.user_id(),
        )
        .unwrap();

    assert!(matches!(
        action2.command_list.commands[0].command.as_ref().unwrap(),
        Command::LoadScene(_)
    ));

    session.connect(session.user_id()).unwrap();
    session.connect(session.opponent_id()).unwrap();
}
