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
use protos::spelldawn::game_object_identifier::Id;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{
    ClientRoomLocation, GainManaAction, InitiateRaidAction, ObjectPositionIdentity,
    ObjectPositionIdentityContainer, ObjectPositionRaid, ObjectPositionRoom, PlayerName,
};
use test_utils::client::HasText;
use test_utils::{test_games, *};

#[test]
fn initiate_raid() {
    let (mut g, ids) = test_games::simple_game(
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

    assert_eq!(
        g.user.data.object_index_position(Id::CardId(ids.scheme_id)),
        (0, Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        g.opponent.data.object_index_position(Id::CardId(ids.scheme_id)),
        (0, Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        g.user.data.object_index_position(Id::CardId(ids.minion_id)),
        (1, Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        g.opponent.data.object_index_position(Id::CardId(ids.minion_id)),
        (1, Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        g.user.data.object_index_position(Id::Identity(PlayerName::User.into())),
        (2, Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        g.opponent.data.object_index_position(Id::Identity(PlayerName::Opponent.into())),
        (2, Position::Raid(ObjectPositionRaid {}))
    );

    assert_commands_match_lists(
        &response,
        vec![
            "UpdateGameView",
            "MoveGameObjects",
            "MoveGameObjects",
            "MoveGameObjects",
            "RenderInterface",
        ],
        vec![
            "VisitRoom",
            "Delay",
            "UpdateGameView",
            "MoveGameObjects",
            "MoveGameObjects",
            "MoveGameObjects",
            "RenderInterface",
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
    assert!(!g.user.cards.get(ids.minion_id).revealed_to_me());
    let response = g.click_on(g.opponent_id(), "Activate");
    assert_eq!(g.opponent.this_player.mana(), 97); // Minion costs 3 to summon
    assert!(g.user.cards.get(ids.minion_id).revealed_to_me());
    assert!(g.opponent.cards.get(ids.minion_id).revealed_to_me());
    assert_eq!(PlayerName::User, g.user.data.priority());
    assert_eq!(PlayerName::Opponent, g.opponent.data.priority());
    assert!(g.opponent.interface.main_controls().has_text("Waiting"));
    assert!(g.user.interface.main_controls().has_text("Test Weapon"));
    assert!(g.user.interface.main_controls().has_text("1\u{f06d}"));
    assert!(g.user.interface.main_controls().has_text("Continue"));
    assert_eq!(
        g.user.data.object_index_position(Id::CardId(ids.scheme_id)),
        (0, Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        g.user.data.object_index_position(Id::CardId(ids.minion_id)),
        (1, Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        g.user.data.object_index_position(Id::Identity(PlayerName::User.into())),
        (2, Position::Raid(ObjectPositionRaid {}))
    );

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

#[test]
fn use_weapon() {
    let (mut g, ids) = test_games::simple_game(
        Side::Champion,
        Some(1004),
        CardName::TestScheme31,
        CardName::TestMinion5Health,
        CardName::TestWeapon3Attack12Boost,
    );
    g.perform(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    g.perform_click_on(g.opponent_id(), "Activate");
    assert_eq!(g.user.this_player.mana(), 97); // Minion costs 3 to summon
    let response = g.click_on(g.user_id(), "Test Weapon");
    assert_eq!(g.user.this_player.mana(), 96); // Weapon costs 1 to use
    assert_eq!(g.opponent.other_player.mana(), 96); // Weapon costs 1 to use
    assert!(g.user.cards.get(ids.scheme_id).revealed_to_me());
    assert!(g.opponent.cards.get(ids.scheme_id).revealed_to_me());
    assert_eq!(PlayerName::User, g.user.data.priority());
    assert_eq!(PlayerName::Opponent, g.opponent.data.priority());
    assert!(g.opponent.interface.main_controls().has_text("Waiting"));
    assert!(g.user.interface.card_anchor_nodes().has_text("Score!"));
    assert!(g.user.interface.main_controls().has_text("End Raid"));

    assert_eq!(
        g.user.data.object_index_position(Id::CardId(ids.scheme_id)),
        (0, Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        g.user.data.object_position(Id::CardId(ids.minion_id)),
        Position::Room(ObjectPositionRoom {
            room_id: CLIENT_ROOM_ID.into(),
            room_location: ClientRoomLocation::Front.into()
        })
    );
    assert_eq!(
        g.user.data.object_index_position(Id::Identity(PlayerName::User.into())),
        (1, Position::Raid(ObjectPositionRaid {}))
    );

    assert_commands_match_lists(
        &response,
        vec![
            "FireProjectile",
            "UpdateGameView",
            "CreateOrUpdateCard", // Reveal card
            "MoveGameObjects",    // Move identity
            "MoveGameObjects",    // Move minion
            "RenderInterface",
        ],
        vec![
            "FireProjectile",
            "UpdateGameView",
            "CreateOrUpdateCard", // Reveal card
            "MoveGameObjects",    // Move identity
            "MoveGameObjects",    // Move minion
        ],
    );

    assert_debug_snapshot!(response);
}

#[test]
fn fire_combat_ability() {
    let (mut g, ids) = test_games::simple_game(
        Side::Champion,
        Some(1006),
        CardName::TestScheme31,
        CardName::TestMinion5Health,
        CardName::TestWeapon3Attack12Boost,
    );
    g.perform(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    g.perform_click_on(g.opponent_id(), "Activate");
    assert_eq!(g.user.this_player.mana(), 97); // Minion costs 3 to summon
    let response = g.click_on(g.user_id(), "Continue");
    assert_eq!(g.user.this_player.mana(), 97); // Mana is unchanged
    assert_eq!(g.opponent.other_player.mana(), 97);
    assert!(!g.user.cards.get(ids.scheme_id).revealed_to_me()); // Scheme is not revealed
    assert_eq!(PlayerName::User, g.user.data.priority()); // Still Champion turn
    assert_eq!(PlayerName::Opponent, g.opponent.data.priority());
    assert!(!g.user.data.raid_active()); // No raid active due to End Raid ability
    assert!(!g.opponent.data.raid_active());

    assert_eq!(
        g.user.data.object_position(Id::CardId(ids.minion_id)),
        Position::Room(ObjectPositionRoom {
            room_id: CLIENT_ROOM_ID.into(),
            room_location: ClientRoomLocation::Front.into()
        })
    );
    assert_eq!(
        g.user.data.object_position(Id::CardId(ids.scheme_id)),
        Position::Room(ObjectPositionRoom {
            room_id: CLIENT_ROOM_ID.into(),
            room_location: ClientRoomLocation::Back.into()
        })
    );
    assert_eq!(
        g.user.data.object_position(Id::Identity(PlayerName::User.into())),
        Position::IdentityContainer(ObjectPositionIdentityContainer {
            owner: PlayerName::User.into()
        })
    );

    assert_commands_match(
        &response,
        vec![
            "FireProjectile",
            "UpdateGameView",
            "MoveGameObjects",
            "MoveGameObjects",
            "MoveGameObjects",
            "RenderInterface",
        ],
    );

    assert_debug_snapshot!(response);
}

#[test]
fn score_scheme_card() {
    let (mut g, ids) = test_games::simple_game(
        Side::Champion,
        Some(1008),
        CardName::TestScheme31,
        CardName::TestMinion5Health,
        CardName::TestWeapon3Attack12Boost,
    );
    g.perform(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    g.perform_click_on(g.opponent_id(), "Activate");
    g.perform_click_on(g.user_id(), "Test Weapon");
    let response = g.click_on(g.user_id(), "Score");

    assert_eq!(g.user.this_player.score(), 1);
    assert_eq!(g.opponent.other_player.score(), 1);
    assert_eq!(PlayerName::User, g.user.data.priority());
    assert_eq!(PlayerName::Opponent, g.opponent.data.priority());
    assert!(g.user.data.raid_active()); // Raid still active
    assert!(g.opponent.data.raid_active());
    assert!(g.opponent.interface.main_controls().has_text("Waiting"));
    assert!(g.user.interface.main_controls().has_text("End Raid"));

    assert_eq!(
        g.user.data.object_position(Id::CardId(ids.scheme_id)),
        Position::Identity(ObjectPositionIdentity { owner: PlayerName::User.into() })
    );
    assert_eq!(
        g.user.data.object_index_position(Id::Identity(PlayerName::User.into())),
        (0, Position::Raid(ObjectPositionRaid {}))
    );

    assert_commands_match_lists(
        &response,
        vec![
            "SetMusic",
            "PlaySound",
            "MoveGameObjects",
            "PlayEffect",
            "PlayEffect",
            "UpdateGameView",
            "MoveGameObjects",
            "MoveGameObjects",
            "MoveGameObjects",
            "RenderInterface",
        ],
        vec![
            "SetMusic",
            "PlaySound",
            "MoveGameObjects",
            "PlayEffect",
            "PlayEffect",
            "UpdateGameView",
            "MoveGameObjects",
            "MoveGameObjects",
            "MoveGameObjects",
        ],
    );

    assert_debug_snapshot!(response);
}

#[test]
fn complete_raid() {
    let (mut g, ids) = test_games::simple_game(
        Side::Champion,
        Some(1010),
        CardName::TestScheme31,
        CardName::TestMinion5Health,
        CardName::TestWeapon3Attack12Boost,
    );
    // Gain mana to spend an action point. Should be Overlord turn after this raid.
    g.perform(Action::GainMana(GainManaAction {}), g.user_id());
    g.perform(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    g.perform_click_on(g.opponent_id(), "Activate");
    g.perform_click_on(g.user_id(), "Test Weapon");
    g.perform_click_on(g.user_id(), "Score");
    let response = g.click_on(g.user_id(), "End Raid");

    assert_eq!(g.user.this_player.score(), 1);
    assert_eq!(g.opponent.other_player.score(), 1);
    assert_eq!(PlayerName::Opponent, g.user.data.priority());
    assert_eq!(PlayerName::User, g.opponent.data.priority());
    assert_eq!(g.opponent.interface.main_controls_option(), None);
    assert_eq!(g.user.interface.main_controls_option(), None);
    assert!(!g.user.data.raid_active()); // Raid no longer active
    assert!(!g.opponent.data.raid_active());

    assert_eq!(
        g.user.data.object_position(Id::CardId(ids.scheme_id)),
        Position::Identity(ObjectPositionIdentity { owner: PlayerName::User.into() })
    );
    assert_eq!(
        g.user.data.object_position(Id::Identity(PlayerName::User.into())),
        Position::IdentityContainer(ObjectPositionIdentityContainer {
            owner: PlayerName::User.into()
        })
    );

    assert_commands_match(
        &response,
        vec!["UpdateGameView", "MoveGameObjects", "RenderInterface", "DisplayGameMessage"],
    );

    assert_debug_snapshot!(response);
}
