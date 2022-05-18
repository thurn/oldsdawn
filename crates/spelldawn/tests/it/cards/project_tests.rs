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
use protos::spelldawn::PlayerName;
use test_utils::*;

#[test]
fn gold_mine() {
    let mut g = new_game(Side::Overlord, Args::default());
    let id = g.play_from_hand(CardName::GoldMine);
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    assert!(g.dawn());
    assert_eq!(STARTING_MANA, g.me().mana());
    spend_actions_until_turn_over(&mut g, Side::Champion);
    assert!(g.dusk());
    assert_eq!(STARTING_MANA - 4 /* cost */ + 3 /* taken */, g.me().mana());
    assert_eq!("9", g.user.get_card(id).arena_icon());
}

#[test]
fn gemcarver() {
    let (card_cost, taken) = (2, 3);
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::Gemcarver);
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    spend_actions_until_turn_over(&mut g, Side::Champion);
    assert_eq!(STARTING_MANA - card_cost + taken, g.me().mana());
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    spend_actions_until_turn_over(&mut g, Side::Champion);
    assert_eq!(STARTING_MANA - card_cost + taken * 2, g.me().mana());
    assert_eq!(2, g.user.cards.hand(PlayerName::User).len());
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    spend_actions_until_turn_over(&mut g, Side::Champion);
    assert_eq!(STARTING_MANA - card_cost + taken * 3, g.me().mana());
    assert_eq!(4, g.user.cards.hand(PlayerName::User).len());
}
