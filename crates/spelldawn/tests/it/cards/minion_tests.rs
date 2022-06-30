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
use ui_core::icons;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{ClientRoomLocation, ObjectPositionRaid, PlayerName};
use test_utils::client::HasText;
use test_utils::*;

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
fn time_golem_pay_mana() {
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

#[test]
fn temporal_vortex_end_raid() {
    let mut g = new_game(Side::Overlord, Args::default());
    g.add_to_hand(CardName::TestMinionEndRaid);
    g.play_from_hand(CardName::TemporalVortex);
    set_up_minion_combat(&mut g);
    assert_eq!(1, g.user.cards.hand(PlayerName::User).len());
    g.click_on(g.opponent_id(), "End Raid");
    assert!(!g.user.data.raid_active());
    assert_eq!(
        vec!["Temporal Vortex", "Test Minion End Raid"],
        g.user.cards.room_cards(ROOM_ID, ClientRoomLocation::Front)
    );
    assert_eq!(0, g.user.cards.hand(PlayerName::User).len());
    assert_eq!(2, g.opponent.this_player.actions());
}

#[test]
fn temporal_vortex_pay_actions() {
    let mut g = new_game(Side::Overlord, Args::default());
    g.add_to_hand(CardName::TestMinionEndRaid);
    g.play_from_hand(CardName::TemporalVortex);
    set_up_minion_combat(&mut g);
    g.click_on(g.opponent_id(), format!("Pay 2{}", icons::ACTION));
    assert_eq!(0, g.opponent.this_player.actions());
    assert!(g.user.data.raid_active());
    assert_eq!(
        vec!["Test Minion End Raid", "Test Scheme 31"],
        g.user.cards.names_in_position(Position::Raid(ObjectPositionRaid {}))
    );
    assert_eq!(
        vec!["Temporal Vortex"],
        g.user.cards.room_cards(ROOM_ID, ClientRoomLocation::Front)
    );
    assert_eq!(0, g.user.cards.hand(PlayerName::User).len());
    assert!(g.opponent.interface.controls().has_text("Advance"));
    assert!(g.opponent.interface.controls().has_text("Retreat"));
}

#[test]
fn temporal_vortex_defeat() {
    let mut g = new_game(Side::Overlord, Args::default());
    g.add_to_hand(CardName::TestMinionEndRaid);
    g.play_from_hand(CardName::TemporalVortex);
    set_up_minion_combat_with_action(&mut g, |g| {
        g.play_from_hand(CardName::TestWeaponAbyssal);
    });
    g.click_on(g.opponent_id(), "Test Weapon");
    assert_eq!(1, g.user.cards.hand(PlayerName::User).len());
    assert_eq!(
        vec!["Temporal Vortex"],
        g.user.cards.room_cards(ROOM_ID, ClientRoomLocation::Front)
    );
    assert!(g.opponent.interface.controls().has_text("Score"));
}

#[test]
fn shadow_lurker_outer_room() {
    let mut g = new_game(Side::Overlord, Args::default());
    let id = g.add_to_hand(CardName::ShadowLurker);
    assert_eq!("2", g.user.get_card(id).bottom_right_icon());
    let id = g.play_from_hand(CardName::ShadowLurker);
    assert_eq!("4", g.user.get_card(id).bottom_right_icon());
    set_up_minion_combat_with_action(&mut g, |g| {
        g.play_from_hand(CardName::TestWeaponAbyssal);
    });
    g.click_on(g.opponent_id(), "Test Weapon");
    assert_eq!(STARTING_MANA - 5, g.opponent.this_player.mana());
}

#[test]
fn shadow_lurker_inner_room() {
    let mut g = new_game(Side::Overlord, Args::default());
    let id = g.play_with_target_room(CardName::ShadowLurker, RoomId::Sanctum);
    assert_eq!("2", g.user.get_card(id).bottom_right_icon());
}

#[test]
fn sphinx_of_winters_breath_discard_even() {
    let mut g = new_game(
        Side::Overlord,
        Args { opponent_deck_top: Some(CardName::Test0CostChampionSpell), ..Args::default() },
    );
    g.play_from_hand(CardName::SphinxOfWintersBreath);
    set_up_minion_combat_with_action(&mut g, |g| {
        g.add_to_hand(CardName::Test0CostChampionSpell);
    });
    click_on_continue(&mut g);
    assert_eq!(vec!["Test 0 Cost Champion Spell"], g.opponent.cards.discard_pile(PlayerName::User));
    assert!(g.user.data.raid_active());
}

#[test]
fn sphinx_of_winters_breath_discard_odd() {
    let mut g = new_game(
        Side::Overlord,
        Args { opponent_deck_top: Some(CardName::Test1CostChampionSpell), ..Args::default() },
    );
    g.play_from_hand(CardName::SphinxOfWintersBreath);
    set_up_minion_combat_with_action(&mut g, |g| {
        g.add_to_hand(CardName::Test1CostChampionSpell);
    });
    click_on_continue(&mut g);
    assert_eq!(vec!["Test 1 Cost Champion Spell"], g.opponent.cards.discard_pile(PlayerName::User));
    assert!(!g.user.data.raid_active());
}

#[test]
fn bridge_troll_continue() {
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::BridgeTroll);
    set_up_minion_combat(&mut g);
    click_on_continue(&mut g);
    assert!(g.user.data.raid_active());
    assert_eq!(STARTING_MANA - 3, g.opponent.this_player.mana());
}

#[test]
fn bridge_troll_end_raid() {
    let mut g = new_game(Side::Overlord, Args { opponent_mana: 2, ..Args::default() });
    g.play_from_hand(CardName::BridgeTroll);
    set_up_minion_combat(&mut g);
    click_on_continue(&mut g);
    assert!(!g.user.data.raid_active());
    assert_eq!(0, g.opponent.this_player.mana());
}

#[test]
fn stormcaller_take_2() {
    let mut g = new_game(Side::Overlord, Args { opponent_hand_size: 5, ..Args::default() });
    g.play_from_hand(CardName::Stormcaller);
    set_up_minion_combat(&mut g);
    g.click_on(g.opponent_id(), "Take 2");
    assert!(!g.user.data.raid_active());
    assert_eq!(2, g.opponent.cards.discard_pile(PlayerName::User).len());
}

#[test]
fn stormcaller_take_4() {
    let mut g = new_game(Side::Overlord, Args { opponent_hand_size: 5, ..Args::default() });
    g.play_from_hand(CardName::Stormcaller);
    set_up_minion_combat(&mut g);
    g.click_on(g.opponent_id(), "Take 4");
    assert!(g.user.data.raid_active());
    assert_eq!(4, g.opponent.cards.discard_pile(PlayerName::User).len());
}

#[test]
fn stormcaller_take_2_game_over() {
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::Stormcaller);
    set_up_minion_combat(&mut g);
    assert!(!g.opponent.interface.controls().has_text("Take 4"));
    g.click_on(g.opponent_id(), "Take 2");
    assert!(g.is_victory_for_player(Side::Overlord));
}

#[test]
fn fire_goblin() {
    let (cost, gained) = (1, 1);
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::FireGoblin);
    set_up_minion_combat(&mut g);
    assert_eq!(STARTING_MANA - cost, g.me().mana());
    click_on_continue(&mut g);
    assert_eq!(STARTING_MANA - cost + gained, g.me().mana());
    assert_eq!(1, g.opponent.cards.discard_pile(PlayerName::User).len());
}
