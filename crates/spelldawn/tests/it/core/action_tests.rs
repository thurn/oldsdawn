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

use cards::test_cards::{ARTIFACT_COST, MANA_STORED, MANA_TAKEN, UNVEIL_COST};
use data::card_name::CardName;
use data::game::RaidPhase;
use data::primitives::Side;
use insta::assert_snapshot;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{
    card_target, CardTarget, ClientRoomLocation, DrawCardAction, GainManaAction, GameMessageType,
    LevelUpRoomAction, ObjectPositionDiscardPile, PlayCardAction, PlayerName,
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
        Args { turn_actions: 3, deck_top: Some(CardName::IceDragon), ..Args::default() },
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
        Args { turn_actions: 3, deck_top: Some(CardName::IceDragon), ..Args::default() },
    );
    let response = g.perform_action(Action::DrawCard(DrawCardAction {}), g.user_id());
    assert_snapshot!(Summary::run(&response));

    assert_identical(vec![CardName::IceDragon], g.user.cards.hand(PlayerName::User));
    assert_eq!(vec![HIDDEN_CARD], g.opponent.cards.hand(PlayerName::Opponent));
    assert_eq!(2, g.me().actions());
    assert_eq!(2, g.opponent.other_player.actions());
}

#[test]
fn cannot_draw_card_on_opponent_turn() {
    let mut g = new_game(Side::Overlord, Args::default());
    assert_error(g.perform_action(Action::DrawCard(DrawCardAction {}), g.opponent_id()));
}

#[test]
fn cannot_draw_when_out_of_action_points() {
    let mut g = new_game(Side::Overlord, Args { turn_actions: 0, ..Args::default() });
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
    let mut g = new_game(Side::Champion, Args { turn_actions: 3, mana: 5, ..Args::default() });
    let card_id = g.add_to_hand(CardName::ArcaneRecovery);
    let response = g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(card_id), target: None }),
        g.user_id(),
    );
    assert_snapshot!(Summary::run(&response));

    assert_eq!(2, g.me().actions());
    assert_eq!(2, g.opponent.other_player.actions());
    assert_eq!(9, g.me().mana());
    assert_eq!(9, g.opponent.other_player.mana());
    assert_identical(vec![CardName::ArcaneRecovery], g.user.cards.discard_pile(PlayerName::User));
    assert_identical(
        vec![CardName::ArcaneRecovery],
        g.opponent.cards.discard_pile(PlayerName::Opponent),
    );
}

#[test]
fn play_hidden_card() {
    let mut g = new_game(Side::Overlord, Args { turn_actions: 3, mana: 0, ..Args::default() });
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
    assert_snapshot!(Summary::run(&response));

    assert_eq!(2, g.me().actions());
    assert_eq!(2, g.opponent.other_player.actions());
    assert_eq!(0, g.me().mana());
    assert_eq!(0, g.opponent.other_player.mana());
    assert_identical(
        vec![CardName::DungeonAnnex],
        g.user.cards.room_cards(ROOM_ID, ClientRoomLocation::Back),
    );
    assert_eq!(vec![HIDDEN_CARD], g.opponent.cards.room_cards(ROOM_ID, ClientRoomLocation::Back));
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
    let mut g = new_game(Side::Champion, Args { turn_actions: 0, ..Args::default() });
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
    let mut g = new_game(Side::Overlord, Args { turn_actions: 3, mana: 5, ..Args::default() });
    let response = g.perform_action(Action::GainMana(GainManaAction {}), g.user_id());

    assert_eq!(2, g.me().actions());
    assert_eq!(2, g.opponent.other_player.actions());
    assert_eq!(6, g.me().mana());
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
    let mut g = new_game(Side::Overlord, Args { turn_actions: 0, ..Args::default() });
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
fn level_up_room() {
    let mut g = new_game(Side::Overlord, Args { mana: 10, ..Args::default() });
    g.play_from_hand(CardName::TestScheme31);
    let response = g.perform_action(
        Action::LevelUpRoom(LevelUpRoomAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );

    assert_snapshot!(Summary::run(&response));
    assert_eq!(g.user.this_player.mana(), 9);
    assert_eq!(g.opponent.other_player.mana(), 9);
}

#[test]
fn score_overlord_card() {
    let mut g = new_game(Side::Overlord, Args { mana: 10, turn_actions: 5, ..Args::default() });
    let scheme_id = g.play_from_hand(CardName::TestScheme31);
    let level_up = Action::LevelUpRoom(LevelUpRoomAction { room_id: CLIENT_ROOM_ID.into() });
    g.perform(level_up.clone(), g.user_id());
    g.perform(level_up.clone(), g.user_id());
    let response = g.perform_action(level_up, g.user_id());

    assert_snapshot!(Summary::run(&response));
    assert!(g.opponent.cards.get(scheme_id).revealed_to_me());
    assert_eq!(g.user.this_player.mana(), 7);
    assert_eq!(g.opponent.other_player.mana(), 7);
    assert_eq!(g.user.this_player.score(), 1);
    assert_eq!(g.opponent.other_player.score(), 1);
}

#[test]
fn overlord_win_game() {
    let mut g =
        new_game(Side::Overlord, Args { mana: 10, score: 6, turn_actions: 5, ..Args::default() });
    g.play_from_hand(CardName::TestScheme31);
    let level_up = Action::LevelUpRoom(LevelUpRoomAction { room_id: CLIENT_ROOM_ID.into() });
    g.perform(level_up.clone(), g.user_id());
    g.perform(level_up.clone(), g.user_id());
    let response = g.perform_action(level_up, g.user_id());

    assert_snapshot!(Summary::run(&response));
    assert_eq!(g.user.data.last_message(), GameMessageType::Victory);
    assert_eq!(g.opponent.data.last_message(), GameMessageType::Defeat);
}

#[test]
fn switch_turn() {
    let mut g = new_game(Side::Overlord, Args { turn_actions: 3, mana: 5, ..Args::default() });
    g.perform_action(Action::GainMana(GainManaAction {}), g.user_id()).unwrap();
    g.perform_action(Action::GainMana(GainManaAction {}), g.user_id()).unwrap();
    let response = g.perform_action(Action::GainMana(GainManaAction {}), g.user_id());
    assert_snapshot!(Summary::run(&response));

    assert_eq!(8, g.me().mana());
    assert_eq!(8, g.opponent.other_player.mana());
    assert_eq!(0, g.me().actions());
    assert_eq!(0, g.opponent.other_player.actions());
    assert_eq!(3, g.user.other_player.actions());
    assert_eq!(3, g.opponent.this_player.actions());
    assert_eq!(g.user.data.priority(), PlayerName::Opponent);
    assert_eq!(g.opponent.data.priority(), PlayerName::User);
}

#[test]
fn activate_ability() {
    let mut g = new_game(Side::Champion, Args { turn_actions: 3, ..Args::default() });
    g.play_from_hand(CardName::TestActivatedAbilityTakeMana);
    let ability_card_id = g
        .user
        .cards
        .cards_in_hand(PlayerName::User)
        .find(|c| c.id().ability_id.is_some())
        .expect("ability card")
        .id();

    assert_eq!(STARTING_MANA - ARTIFACT_COST, g.me().mana());
    assert_eq!(2, g.me().actions());

    let response = g.perform_action(
        Action::PlayCard(PlayCardAction { card_id: Some(ability_card_id), target: None }),
        g.user_id(),
    );

    assert_snapshot!(Summary::run(&response));
    assert_eq!(STARTING_MANA - ARTIFACT_COST + MANA_TAKEN, g.me().mana());
    assert_eq!(1, g.me().actions());
}

#[test]
fn activate_ability_take_all_mana() {
    let mut g = new_game(Side::Champion, Args { turn_actions: 3, ..Args::default() });
    let id = g.play_from_hand(CardName::TestActivatedAbilityTakeMana);
    let ability_card_id = g
        .user
        .cards
        .cards_in_hand(PlayerName::User)
        .find(|c| c.id().ability_id.is_some())
        .expect("ability card")
        .id();

    let mut taken = 0;
    while taken < MANA_STORED {
        g.perform(
            Action::PlayCard(PlayCardAction { card_id: Some(ability_card_id), target: None }),
            g.user_id(),
        );
        taken += MANA_TAKEN;

        draw_cards_until_turn_over(&mut g, Side::Champion);
        assert!(g.dusk());
        draw_cards_until_turn_over(&mut g, Side::Overlord);
        assert!(g.dawn());
    }

    assert_eq!(STARTING_MANA - ARTIFACT_COST + MANA_STORED, g.user.this_player.mana());
    assert_eq!(STARTING_MANA - ARTIFACT_COST + MANA_STORED, g.opponent.other_player.mana());
    assert_eq!(
        Position::DiscardPile(ObjectPositionDiscardPile { owner: PlayerName::User.into() }),
        g.user.cards.get(id).position()
    );
    assert_eq!(
        Position::DiscardPile(ObjectPositionDiscardPile { owner: PlayerName::Opponent.into() }),
        g.opponent.cards.get(id).position()
    );
}

#[test]
fn triggered_ability() {
    let mut g = new_game(Side::Overlord, Args { turn_actions: 1, ..Args::default() });
    g.play_from_hand(CardName::TestTriggeredAbilityTakeManaAtDusk);
    assert!(g.dawn());
    assert_eq!(STARTING_MANA, g.user.this_player.mana());
    g.perform(Action::GainMana(GainManaAction {}), g.opponent_id());
    g.perform(Action::GainMana(GainManaAction {}), g.opponent_id());
    let response = g.perform_action(Action::GainMana(GainManaAction {}), g.opponent_id());
    assert!(g.dusk());
    assert_snapshot!(Summary::run(&response));
    assert_eq!(STARTING_MANA - UNVEIL_COST + MANA_TAKEN, g.user.this_player.mana());
    assert_eq!(STARTING_MANA - UNVEIL_COST + MANA_TAKEN, g.opponent.other_player.mana());
}

#[test]
fn triggered_ability_cannot_unveil() {
    let mut g = new_game(Side::Overlord, Args { turn_actions: 1, mana: 0, ..Args::default() });
    g.play_from_hand(CardName::TestTriggeredAbilityTakeManaAtDusk);
    assert!(g.dawn());
    assert_eq!(0, g.user.this_player.mana());
    gain_mana_until_turn_over(&mut g, Side::Champion);
    assert!(g.dusk());
    assert_eq!(0, g.user.this_player.mana());
    assert_eq!(0, g.opponent.other_player.mana());
}

#[test]
fn triggered_ability_take_all_mana() {
    let mut g = new_game(Side::Overlord, Args { turn_actions: 1, ..Args::default() });
    let id = g.play_from_hand(CardName::TestTriggeredAbilityTakeManaAtDusk);
    let mut taken = 0;
    while taken < MANA_STORED {
        assert!(g.dawn());
        gain_mana_until_turn_over(&mut g, Side::Champion);
        assert!(g.dusk());
        taken += MANA_TAKEN;
        draw_cards_until_turn_over(&mut g, Side::Overlord);
    }

    assert_eq!(STARTING_MANA - UNVEIL_COST + MANA_STORED, g.user.this_player.mana());
    assert_eq!(STARTING_MANA - UNVEIL_COST + MANA_STORED, g.opponent.other_player.mana());
    assert_eq!(
        Position::DiscardPile(ObjectPositionDiscardPile { owner: PlayerName::User.into() }),
        g.user.cards.get(id).position()
    );
    assert_eq!(
        Position::DiscardPile(ObjectPositionDiscardPile { owner: PlayerName::Opponent.into() }),
        g.opponent.cards.get(id).position()
    );
}
