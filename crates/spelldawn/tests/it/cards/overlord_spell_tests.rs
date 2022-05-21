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
use test_utils::*;

#[test]
fn gathering_dark() {
    let (cost, gained) = (5, 9);
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::GatheringDark);
    assert_eq!(STARTING_MANA - cost + gained, g.me().mana());
}

#[test]
fn overwhelming_power() {
    let (cost, gained) = (10, 15);
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::OverwhelmingPower);
    assert_eq!(STARTING_MANA - cost + gained, g.me().mana());
}

#[test]
fn forced_march() {
    let mut g = new_game(Side::Overlord, Args::default());
    let scheme = g.play_from_hand(CardName::TestScheme31);
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    spend_actions_until_turn_over(&mut g, Side::Champion);
    g.play_with_target_room(CardName::ForcedMarch, ROOM_ID);
    assert_eq!("2", g.user.get_card(scheme).arena_icon());
}

#[test]
#[should_panic]
fn forced_march_same_turn_panic() {
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::TestScheme31);
    g.play_with_target_room(CardName::ForcedMarch, ROOM_ID);
}
