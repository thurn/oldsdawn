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

use cards::test_cards::{MINION_COST, MINION_HEALTH, TEST_FACTION};
use data::card_name::CardName;
use data::primitives::{Faction, RoomId, Side};
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{ObjectPositionBrowser, PlayerName};
use test_utils::*;

#[test]
fn arcane_recovery() {
    let mut g = new_game(Side::Champion, Args { mana: 5, ..Args::default() });
    g.play_from_hand(CardName::ArcaneRecovery);
    assert_eq!(9, g.me().mana());
    assert_eq!(9, g.opponent.other_player.mana())
}

#[test]
fn meditation() {
    let mut g = new_game(Side::Champion, Args { mana: 5, ..Args::default() });
    assert_eq!(3, g.me().actions());
    g.play_from_hand(CardName::Meditation);
    assert_eq!(9, g.me().mana());
    assert_eq!(1, g.me().actions());
    g.play_from_hand(CardName::Meditation);
    assert_eq!(13, g.me().mana());
    assert!(g.dusk());
}

#[test]
fn coup_de_grace() {
    let mut g = new_game(Side::Champion, Args::default());
    g.play_with_target_room(CardName::CoupDeGrace, RoomId::Vault);
    assert!(g.user.data.raid_active());
    assert_eq!(2, g.user.cards.in_position(Position::Browser(ObjectPositionBrowser {})).count());
    assert_eq!(0, g.user.cards.hand(PlayerName::User).len());
    g.click_on(g.user_id(), "End Raid");
    assert_eq!(1, g.user.cards.hand(PlayerName::User).len());
}

#[test]
#[should_panic]
fn coup_de_grace_invalid_room() {
    let mut g = new_game(Side::Champion, Args::default());
    g.play_with_target_room(CardName::CoupDeGrace, ROOM_ID);
}

#[test]
fn charged_strike() {
    let mut g = new_game(Side::Champion, Args::default());
    setup_raid_target(&mut g, TEST_FACTION);
    g.play_from_hand(CardName::TestWeapon3Attack12Boost3Cost);
    assert_eq!(STARTING_MANA - 3, g.me().mana());
    g.play_with_target_room(CardName::ChargedStrike, ROOM_ID);
    assert!(g.user.data.raid_active());
    assert_eq!(STARTING_MANA - 4, g.me().mana());
    assert_eq!(5, g.user.this_player.bonus_mana());
    assert_eq!(5, g.opponent.other_player.bonus_mana());
    click_on_activate(&mut g);
    g.click_on(g.user_id(), "Test Weapon");
    assert_eq!(STARTING_MANA - 4, g.me().mana());
    assert_eq!(4, g.user.this_player.bonus_mana());
    assert_eq!(4, g.opponent.other_player.bonus_mana());
}

#[test]
fn stealth_mission() {
    let mut g = new_game(Side::Champion, Args::default());
    setup_raid_target(&mut g, TEST_FACTION);
    g.play_with_target_room(CardName::StealthMission, ROOM_ID);
    assert_eq!(STARTING_MANA, g.opponent.this_player.mana());
    click_on_activate(&mut g);
    assert_eq!(STARTING_MANA - MINION_COST - 3, g.opponent.this_player.mana());
}
