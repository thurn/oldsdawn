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
use protos::spelldawn::{DrawCardAction, GainManaAction, PlayCardAction, PlayerName};
use test_utils::*;

#[test]
fn draw_card() {
    let mut g = new_game(
        Side::Overlord,
        Args { actions: 3, next_draw: Some(CardName::IceDragon), ..Args::default() },
    );
    let response = g.perform_action(Action::DrawCard(DrawCardAction {}), USER_ID);
    assert_identical(vec![CardName::IceDragon], g.hand(PlayerName::User));
    assert_eq!(2, g.user.actions());
    assert_ok(&response);
    assert_debug_snapshot!(response);
}

#[test]
fn cannot_draw_card_on_opponent_turn() {
    let mut g = new_game(Side::Overlord, Args::default());
    assert_error(g.perform_action(Action::DrawCard(DrawCardAction {}), OPPONENT_ID));
}

#[test]
fn cannot_draw_when_out_of_action_points() {
    let mut g = new_game(Side::Overlord, Args { actions: 0, ..Args::default() });
    assert_error(g.perform_action(Action::DrawCard(DrawCardAction {}), USER_ID));
}

#[test]
fn cannot_draw_during_raid() {
    let mut g = new_game(
        Side::Overlord,
        Args { raid: Some(TestRaid { priority: Side::Overlord }), ..Args::default() },
    );
    assert_error(g.perform_action(Action::DrawCard(DrawCardAction {}), USER_ID));
}

#[test]
fn play_card() {
    let mut g = new_game(Side::Champion, Args { actions: 3, mana: 5, ..Args::default() });
    let card_id = g.draw_named_card(CardName::ArcaneRecovery);
    let response = g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        USER_ID,
    );
    assert_eq!(2, g.user.actions());
    assert_eq!(9, g.user.mana());
    assert_identical(vec![CardName::ArcaneRecovery], g.discard_pile(PlayerName::User));
    assert_ok(&response);
    assert_debug_snapshot!(response);
}

#[test]
fn cannot_play_card_on_opponent_turn() {
    let mut g = new_game(Side::Overlord, Args::default());
    let card_id = g.draw_named_card(CardName::ArcaneRecovery);
    assert_error(g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        USER_ID,
    ));
}

#[test]
fn cannot_play_card_when_out_of_action_points() {
    let mut g = new_game(Side::Champion, Args { actions: 0, ..Args::default() });
    let card_id = g.draw_named_card(CardName::ArcaneRecovery);
    assert_error(g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        USER_ID,
    ));
}

#[test]
fn cannot_play_card_during_raid() {
    let mut g = new_game(
        Side::Champion,
        Args { raid: Some(TestRaid { priority: Side::Overlord }), ..Args::default() },
    );
    let card_id = g.draw_named_card(CardName::ArcaneRecovery);
    assert_error(g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        USER_ID,
    ));
}

#[test]
fn gain_mana() {
    let mut g = new_game(Side::Overlord, Args { actions: 3, mana: 5, ..Args::default() });
    let response = g.perform_action(Action::GainMana(GainManaAction {}), USER_ID);
    assert_eq!(2, g.user.actions());
    assert_eq!(6, g.user.mana());
    assert_ok(&response);
    assert_debug_snapshot!(response);
}

#[test]
fn cannot_gain_mana_on_opponent_turn() {
    let mut g = new_game(Side::Overlord, Args::default());
    assert_error(g.perform_action(Action::GainMana(GainManaAction {}), OPPONENT_ID));
}

#[test]
fn cannot_gain_mana_when_out_of_action_points() {
    let mut g = new_game(Side::Overlord, Args { actions: 0, ..Args::default() });
    assert_error(g.perform_action(Action::GainMana(GainManaAction {}), USER_ID));
}

#[test]
fn cannot_gain_mana_during_raid() {
    let mut g = new_game(
        Side::Overlord,
        Args { raid: Some(TestRaid { priority: Side::Overlord }), ..Args::default() },
    );
    assert_error(g.perform_action(Action::GainMana(GainManaAction {}), USER_ID));
}
