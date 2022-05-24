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
use test_utils::client::HasText;
use test_utils::*;
use ui::icons;

#[test]
fn ice_dragon() {
    let mut g = new_game(Side::Overlord, Args { opponent_hand_size: 5, ..Args::default() });
    g.play_from_hand(CardName::IceDragon);
    set_up_minion_combat(&mut g);
    click_on_continue(&mut g);
    assert!(!g.user.data.raid_active());
    assert_eq!(1, g.user.cards.discard_pile(PlayerName::Opponent).len());
    assert_eq!(5, g.user.cards.hand(PlayerName::Opponent).len()); // Card is drawn for turn!
}

#[test]
fn time_golem() {
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::TimeGolem);
    set_up_minion_combat(&mut g);
    assert!(g.opponent.interface.controls().has_text("End Raid"));
    assert!(g.opponent.interface.controls().has_text(format!("Pay 5{}", icons::MANA)));
    assert!(g.opponent.interface.controls().has_text(format!("Pay 2{}", icons::ACTION)));
    g.click_on(g.opponent_id(), format!("Pay 5{}", icons::MANA));
    assert!(g.opponent.interface.controls().has_text("Continue"));
    assert_eq!(STARTING_MANA - 5, g.opponent.this_player.mana());
}

#[test]
fn time_golem_defeat() {
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::TimeGolem);
    g.play_from_hand(CardName::TestScheme31);
    spend_actions_until_turn_over(&mut g, Side::Overlord);
    g.play_from_hand(CardName::TestWeapon5Attack);
    g.initiate_raid(ROOM_ID);
    click_on_activate(&mut g);
    g.click_on(g.opponent_id(), format!("Pay 5{}", icons::MANA));
    g.click_on(g.opponent_id(), "Test Weapon");
    assert_eq!(vec!["Time Golem"], g.user.cards.discard_pile(PlayerName::User));
}

#[test]
fn time_golem_pay_actions() {
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::TimeGolem);
    set_up_minion_combat(&mut g);
    g.click_on(g.opponent_id(), format!("Pay 2{}", icons::ACTION));
    assert_eq!(0, g.opponent.this_player.actions());
    click_on_continue(&mut g);
    click_on_score(&mut g);
    click_on_end_raid(&mut g);
    assert!(g.dusk());
}

#[test]
fn time_golem_end_raid() {
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::TimeGolem);
    set_up_minion_combat(&mut g);
    g.click_on(g.opponent_id(), "End Raid");
    assert_eq!(2, g.opponent.this_player.actions());
    assert!(!g.user.data.raid_active());
}
