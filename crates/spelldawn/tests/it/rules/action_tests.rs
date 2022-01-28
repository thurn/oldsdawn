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
use data::game::RaidPhase;
use data::primitives::Side;
use insta::assert_snapshot;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::{
    card_target, CardTarget, ClientRoomLocation, DrawCardAction, GainManaAction, PlayCardAction,
    PlayerName,
};
use test_utils::summarize::Summary;
use test_utils::*;

#[test]
fn connect() {
    let mut g = new_game(Side::Overlord, Args { connect: false, ..Args::default() });
    let response = g.connect(g.user_id(), Some(g.game_id()));
    let _summary = Summary::run(&response);
    assert_snapshot!(Summary::run(&response));
}

#[test]
fn connect_to_ongoing() {
    let mut g = new_game(
        Side::Overlord,
        Args { actions: 3, deck_top: Some(CardName::IceDragon), ..Args::default() },
    );
    let r1 = g.connect(g.user_id(), Some(g.game_id()));
    assert_ok(&r1);
    let r2 = g.perform_action(Action::DrawCard(DrawCardAction {}), g.user_id());
    assert_identical(vec![CardName::IceDragon], g.user.cards.hand(PlayerName::User));
    assert_ok(&r2);
    let r3 = g.connect(g.opponent_id(), Some(g.game_id()));

    assert_snapshot!(Summary::run(&r3));
}

#[test]
fn draw_card() {
    let mut g = new_game(
        Side::Overlord,
        Args { actions: 3, deck_top: Some(CardName::IceDragon), ..Args::default() },
    );
    let response = g.perform_action(Action::DrawCard(DrawCardAction {}), g.user_id());
    assert_identical(vec![CardName::IceDragon], g.user.cards.hand(PlayerName::User));
    assert_eq!(vec![HIDDEN_CARD], g.opponent.cards.hand(PlayerName::Opponent));
    assert_eq!(2, g.player().actions());
    assert_eq!(2, g.opponent.other_player.actions());

    assert_snapshot!(Summary::run(&response));
}

#[test]
fn cannot_draw_card_on_opponent_turn() {
    let mut g = new_game(Side::Overlord, Args::default());
    assert_error(g.perform_action(Action::DrawCard(DrawCardAction {}), g.opponent_id()));
}

#[test]
fn cannot_draw_when_out_of_action_points() {
    let mut g = new_game(Side::Overlord, Args { actions: 0, ..Args::default() });
    assert_error(g.perform_action(Action::DrawCard(DrawCardAction {}), g.user_id()));
}

#[test]
fn cannot_draw_during_raid() {
    let mut g = new_game(
        Side::Overlord,
        Args {
            raid: Some(TestRaid { phase: RaidPhase::Activation, active: false }),
            ..Args::default()
        },
    );
    assert_error(g.perform_action(Action::DrawCard(DrawCardAction {}), g.user_id()));
}

#[test]
fn play_card() {
    let mut g = new_game(Side::Champion, Args { actions: 3, mana: 5, ..Args::default() });
    let card_id = g.add_to_hand(CardName::ArcaneRecovery);
    let response = g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        g.user_id(),
    );
    assert_eq!(2, g.player().actions());
    assert_eq!(2, g.opponent.other_player.actions());
    assert_eq!(9, g.player().mana());
    assert_eq!(9, g.opponent.other_player.mana());
    assert_identical(vec![CardName::ArcaneRecovery], g.user.cards.discard_pile(PlayerName::User));
    assert_identical(
        vec![CardName::ArcaneRecovery],
        g.opponent.cards.discard_pile(PlayerName::Opponent),
    );

    assert_snapshot!(Summary::run(&response));
}

#[test]
fn play_hidden_card() {
    let mut g = new_game(Side::Overlord, Args { actions: 3, mana: 0, ..Args::default() });
    let card_id = g.add_to_hand(CardName::DungeonAnnex);
    let response = g.perform_action(
        Action::PlayCard(PlayCardAction {
            card_id: Some(card_id),
            target: Some(CardTarget {
                card_target: Some(card_target::CardTarget::RoomId(CLIENT_ROOM_ID.into())),
            }),
        }),
        g.user_id(),
    );
    assert_eq!(2, g.player().actions());
    assert_eq!(2, g.opponent.other_player.actions());
    assert_eq!(0, g.player().mana());
    assert_eq!(0, g.opponent.other_player.mana());
    assert_identical(
        vec![CardName::DungeonAnnex],
        g.user.cards.room_cards(ROOM_ID, ClientRoomLocation::Back),
    );
    assert_eq!(vec![HIDDEN_CARD], g.opponent.cards.room_cards(ROOM_ID, ClientRoomLocation::Back));

    assert_snapshot!(Summary::run(&response));
}

#[test]
fn cannot_play_card_on_opponent_turn() {
    let mut g = new_game(Side::Overlord, Args::default());
    let card_id = g.add_to_hand(CardName::ArcaneRecovery);
    assert_error(g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        g.user_id(),
    ));
}

#[test]
fn cannot_play_card_when_out_of_action_points() {
    let mut g = new_game(Side::Champion, Args { actions: 0, ..Args::default() });
    let card_id = g.add_to_hand(CardName::ArcaneRecovery);
    assert_error(g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        g.user_id(),
    ));
}

#[test]
fn cannot_play_card_during_raid() {
    let mut g = new_game(
        Side::Champion,
        Args {
            raid: Some(TestRaid { phase: RaidPhase::Activation, active: false }),
            ..Args::default()
        },
    );
    let card_id = g.add_to_hand(CardName::ArcaneRecovery);
    assert_error(g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        g.user_id(),
    ));
}

#[test]
fn gain_mana() {
    let mut g = new_game(Side::Overlord, Args { actions: 3, mana: 5, ..Args::default() });
    let response = g.perform_action(Action::GainMana(GainManaAction {}), g.user_id());

    assert_eq!(2, g.player().actions());
    assert_eq!(2, g.opponent.other_player.actions());
    assert_eq!(6, g.player().mana());
    assert_eq!(6, g.opponent.other_player.mana());

    assert_snapshot!(Summary::run(&response));
}

#[test]
fn cannot_gain_mana_on_opponent_turn() {
    let mut g = new_game(Side::Overlord, Args::default());
    assert_error(g.perform_action(Action::GainMana(GainManaAction {}), g.opponent_id()));
}

#[test]
fn cannot_gain_mana_when_out_of_action_points() {
    let mut g = new_game(Side::Overlord, Args { actions: 0, ..Args::default() });
    assert_error(g.perform_action(Action::GainMana(GainManaAction {}), g.user_id()));
}

#[test]
fn cannot_gain_mana_during_raid() {
    let mut g = new_game(
        Side::Overlord,
        Args {
            raid: Some(TestRaid { phase: RaidPhase::Activation, active: false }),
            ..Args::default()
        },
    );
    assert_error(g.perform_action(Action::GainMana(GainManaAction {}), g.user_id()));
}

#[test]
fn switch_turn() {
    let mut g = new_game(Side::Overlord, Args { actions: 3, mana: 5, ..Args::default() });
    g.perform_action(Action::GainMana(GainManaAction {}), g.user_id()).unwrap();
    g.perform_action(Action::GainMana(GainManaAction {}), g.user_id()).unwrap();
    let response = g.perform_action(Action::GainMana(GainManaAction {}), g.user_id());
    assert_eq!(8, g.player().mana());
    assert_eq!(8, g.opponent.other_player.mana());
    assert_eq!(0, g.player().actions());
    assert_eq!(0, g.opponent.other_player.actions());
    assert_eq!(3, g.user.other_player.actions());
    assert_eq!(3, g.opponent.this_player.actions());
    assert_eq!(g.user.data.priority(), PlayerName::Opponent);
    assert_eq!(g.opponent.data.priority(), PlayerName::User);

    assert_snapshot!(Summary::run(&response));
}
