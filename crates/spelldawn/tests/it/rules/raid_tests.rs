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
use data::primitives::Side;
use insta::assert_debug_snapshot;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::{InitiateRaidAction, PlayerName};
use test_utils::client::HasText;
use test_utils::{test_games, *};

#[test]
fn initiate_raid() {
    let (mut g, _) = test_games::simple_game(
        Side::Champion,
        Some(1000),
        CardName::TestScheme31,
        CardName::TestMinion5Health,
        CardName::TestWeapon3Attack12Boost,
    );
    let response = g.perform_action(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    assert_eq!(1, g.player().actions());
    assert_eq!(PlayerName::Opponent, g.user.data.priority());
    assert_eq!(PlayerName::User, g.opponent.data.priority());
    assert!(g.user.data.raid_active());
    assert!(g.opponent.data.raid_active());

    assert_commands_match_lists(
        &response,
        vec![
            "UpdateGameView",
            "RenderInterface",
            "MoveGameObjects",
            "MoveGameObjects",
            "MoveGameObjects",
        ],
        vec![
            "VisitRoom",
            "Delay",
            "UpdateGameView",
            "RenderInterface",
            "MoveGameObjects",
            "MoveGameObjects",
            "MoveGameObjects",
        ],
    );
    assert!(g.user.interface.main_controls().has_text("Waiting"));
    assert!(g.opponent.interface.main_controls().has_text("Activate"));
    assert!(g.opponent.interface.main_controls().has_text("Pass"));
    assert_debug_snapshot!(response);
}

#[test]
fn activate_room() {
    let (mut g, ids) = test_games::simple_game(
        Side::Champion,
        Some(1002),
        CardName::TestScheme31,
        CardName::TestMinion5Health,
        CardName::TestWeapon3Attack12Boost,
    );
    g.perform(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    assert_eq!(g.opponent.this_player.mana(), 100);
    assert!(!g.user.cards.get(&ids.minion_id).revealed_to_me());
    let response = g.click_on(g.opponent_id(), "Activate");
    assert_eq!(g.opponent.this_player.mana(), 97); // Minion costs 3 to summon
    assert!(g.user.cards.get(&ids.minion_id).revealed_to_me());
    assert!(g.opponent.cards.get(&ids.minion_id).revealed_to_me());
    assert_eq!(PlayerName::User, g.user.data.priority());
    assert_eq!(PlayerName::Opponent, g.opponent.data.priority());
    assert!(g.opponent.interface.main_controls().has_text("Waiting"));
    assert!(g.user.interface.main_controls().has_text("Test Weapon"));
    assert!(g.user.interface.main_controls().has_text("1\u{f06d}"));
    assert!(g.user.interface.main_controls().has_text("Continue"));

    assert_commands_match(
        &response,
        vec![
            "UpdateGameView",
            "CreateOrUpdateCard", // Reveal Card
            "RenderInterface",    // Render Prompts
        ],
    );
    assert_debug_snapshot!(response);
}

#[test]
fn activate_room_weapon_2() {
    let (mut g, _) = test_games::simple_game(
        Side::Champion,
        None,
        CardName::TestScheme31,
        CardName::TestMinion5Health,
        CardName::TestWeapon2Attack,
    );
    g.perform(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    g.perform_click_on(g.opponent_id(), "Activate");
    assert!(g.opponent.interface.main_controls().has_text("Waiting"));
    assert!(!g.user.interface.main_controls().has_text("Test Weapon"));
    assert!(g.user.interface.main_controls().has_text("Continue"));
}

#[test]
fn activate_room_weapon_2_12() {
    let (mut g, _) = test_games::simple_game(
        Side::Champion,
        None,
        CardName::TestScheme31,
        CardName::TestMinion5Health,
        CardName::TestWeapon2Attack12Boost,
    );
    g.perform(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    g.perform_click_on(g.opponent_id(), "Activate");
    assert!(g.opponent.interface.main_controls().has_text("Waiting"));
    assert!(g.user.interface.main_controls().has_text("Test Weapon"));
    assert!(g.user.interface.main_controls().has_text("2\u{f06d}"));
    assert!(g.user.interface.main_controls().has_text("Continue"));
}

#[test]
fn activate_room_weapon_4_12() {
    let (mut g, _) = test_games::simple_game(
        Side::Champion,
        None,
        CardName::TestScheme31,
        CardName::TestMinion5Health,
        CardName::TestWeapon4Attack12Boost,
    );
    g.perform(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    g.perform_click_on(g.opponent_id(), "Activate");
    assert!(g.opponent.interface.main_controls().has_text("Waiting"));
    assert!(g.user.interface.main_controls().has_text("Test Weapon"));
    assert!(g.user.interface.main_controls().has_text("1\u{f06d}"));
    assert!(g.user.interface.main_controls().has_text("Continue"));
}

#[test]
fn activate_room_weapon_5() {
    let (mut g, _) = test_games::simple_game(
        Side::Champion,
        None,
        CardName::TestScheme31,
        CardName::TestMinion5Health,
        CardName::TestWeapon5Attack,
    );
    g.perform(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    g.perform_click_on(g.opponent_id(), "Activate");
    assert!(g.opponent.interface.main_controls().has_text("Waiting"));
    assert!(g.user.interface.main_controls().has_text("Test Weapon"));
    assert!(!g.user.interface.main_controls().has_text("\u{f06d}"));
    assert!(g.user.interface.main_controls().has_text("Continue"));
}
