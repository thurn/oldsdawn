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
use protos::spelldawn::RoomIdentifier;
use test_utils::client::HasText;
use test_utils::*;

#[test]
fn lodestone() {
    let mut g = new_game(Side::Champion, Args::default());
    let id = g.play_from_hand(CardName::Lodestone);
    assert_eq!("12", g.user.get_card(id).arena_icon());
    g.activate_ability(id, 1);
    assert_eq!(STARTING_MANA - 1 + 2, g.me().mana());
    assert_eq!(1, g.me().actions());
    assert_eq!("10", g.user.get_card(id).arena_icon());
}

#[test]
fn sanctum_passage() {
    let mut g = new_game(Side::Champion, Args::default());
    g.add_to_hand(CardName::TestScheme31);
    g.add_to_hand(CardName::TestScheme31);

    g.play_from_hand(CardName::SanctumPassage);
    g.initiate_raid(RoomId::Sanctum);
    assert_eq!(2, g.user.interface.card_anchor_nodes().len());
    assert_eq!(vec!["Score!"], g.user.interface.card_anchor_nodes()[0].get_text());
    assert_eq!(vec!["Score!"], g.user.interface.card_anchor_nodes()[1].get_text());
    click_on_end_raid(&mut g);
    g.initiate_raid(RoomId::Sanctum);
    assert_eq!(1, g.user.interface.card_anchor_nodes().len());
    assert_eq!(vec!["Score!"], g.user.interface.card_anchor_nodes()[0].get_text());
}

#[test]
fn accumulator() {
    let card_cost = 3;
    let mut g = new_game(Side::Champion, Args::default());
    let id = g.play_from_hand(CardName::Accumulator);
    g.initiate_raid(RoomId::Crypts);
    click_on_end_raid(&mut g);
    assert_eq!("1", g.user.get_card(id).arena_icon());
    g.activate_ability(id, 1);
    assert_eq!(STARTING_MANA + 2 - card_cost, g.me().mana())
}

#[test]
fn mystic_portal() {
    let card_cost = 5;
    let mut g = new_game(Side::Champion, Args::default());
    let id = g.play_from_hand(CardName::MysticPortal);
    assert_eq!("12", g.user.get_card(id).arena_icon());
    assert_eq!(
        vec![RoomIdentifier::Vault, RoomIdentifier::Sanctum, RoomIdentifier::Crypts],
        g.user.cards.get(ability_id(id, 1)).valid_rooms()
    );
    g.activate_ability_with_target(id, 1, RoomId::Crypts);
    click_on_end_raid(&mut g);
    assert_eq!(STARTING_MANA + 3 - card_cost, g.me().mana());
    assert_eq!("9", g.user.get_card(id).arena_icon());
    assert_eq!(
        vec![RoomIdentifier::Vault, RoomIdentifier::Sanctum],
        g.user.cards.get(ability_id(id, 1)).valid_rooms()
    );
}

#[test]
fn mystic_portal_play_after_raid() {
    let mut g = new_game(Side::Champion, Args::default());
    let id = g.add_to_hand(CardName::MysticPortal);
    g.initiate_raid(RoomId::Sanctum);
    click_on_end_raid(&mut g);
    g.play_card(id, g.user_id(), None);
    assert_eq!("12", g.user.get_card(id).arena_icon());
    assert_eq!(
        vec![RoomIdentifier::Vault, RoomIdentifier::Crypts],
        g.user.cards.get(ability_id(id, 1)).valid_rooms()
    );
}

#[test]
#[should_panic]
fn mystic_portal_repeat_panic() {
    let mut g = new_game(Side::Champion, Args::default());
    let id = g.play_from_hand(CardName::MysticPortal);
    g.activate_ability_with_target(id, 1, RoomId::Crypts);
    click_on_end_raid(&mut g);
    g.activate_ability_with_target(id, 1, RoomId::Crypts);
}
