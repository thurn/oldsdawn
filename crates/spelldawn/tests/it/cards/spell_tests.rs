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
use data::primitives::{RoomId, Side};
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
