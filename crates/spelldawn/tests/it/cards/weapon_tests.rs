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
use data::primitives::{Faction, RoomId, Side};
use test_utils::client::HasText;
use test_utils::*;

#[test]
fn greataxe() {
    let card_cost = 3;
    let ability_cost = 1;
    let mut g = new_game(Side::Champion, Args::default());
    g.play_from_hand(CardName::Greataxe);
    fire_weapon_combat_abilities(&mut g, Faction::Infernal, "Greataxe");
    assert_eq!(STARTING_MANA - card_cost - ability_cost, g.me().mana());
    assert!(g.user.data.raid_active());
    assert!(g.user.interface.controls().has_text("End Raid"));
}

#[test]
fn marauders_axe() {
    let card_cost = 5;
    let mut g = new_game(Side::Champion, Args::default());
    let id = g.add_to_hand(CardName::MaraudersAxe);
    assert_eq!(card_cost.to_string(), g.user.cards.get(id).top_left_icon());
    g.initiate_raid(RoomId::Crypts);
    click_on_end_raid(&mut g);
    assert_eq!((card_cost - 2).to_string(), g.user.cards.get(id).top_left_icon());
    g.play_card(id, g.user_id(), None);
    assert_eq!(STARTING_MANA - card_cost + 2, g.me().mana());
}

#[test]
fn keen_halberd() {
    let (card_cost, activation_cost) = (3, 2);
    let mut g = new_game(Side::Champion, Args::default());
    g.play_from_hand(CardName::KeenHalberd);
    setup_raid_target(&mut g, CardName::TestMinionShield2Abyssal);
    g.initiate_raid(ROOM_ID);
    click_on_activate(&mut g);
    g.click_on(g.user_id(), "Keen Halberd");
    assert_eq!(
        STARTING_MANA - card_cost - (2 * activation_cost) - 1, /* remaining shield */
        g.me().mana()
    );
}
