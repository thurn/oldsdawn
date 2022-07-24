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
use protos::spelldawn::game_action::Action;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{DrawCardAction, ObjectPositionIdentity, PlayerName};
use test_utils::*;

#[test]
fn gold_mine() {
    let mut g = new_game(Side::Overlord, Args::default());
    let id = g.play_from_hand(CardName::GoldMine);
    level_up_room(&mut g, 4);
    assert_eq!(g.me().score(), 2);
    assert_eq!(STARTING_MANA - 4 /* level cost */ + 7 /* gained */, g.me().mana());
    assert_eq!(
        g.user.get_card(id).position(),
        Position::Identity(ObjectPositionIdentity { owner: PlayerName::User.into() })
    );
}

#[test]
fn activate_reinforcements() {
    let mut g = new_game(Side::Overlord, Args::default());
    let id = g.play_from_hand(CardName::ActivateReinforcements);
    let minion = g.play_from_hand(CardName::TestMinionEndRaid);
    assert!(!g.user.get_card(minion).is_face_up());
    level_up_room(&mut g, 5);
    assert_eq!(g.me().score(), 3);
    assert!(g.user.get_card(minion).is_face_up());
    assert_eq!(STARTING_MANA - 5, g.me().mana());
    assert_eq!(
        g.user.get_card(id).position(),
        Position::Identity(ObjectPositionIdentity { owner: PlayerName::User.into() })
    );
}

#[test]
fn research_project() {
    let mut g = new_game(Side::Overlord, Args::default());
    g.play_from_hand(CardName::ResearchProject);
    level_up_room(&mut g, 2);
    spend_actions_until_turn_over(&mut g, Side::Champion);
    assert_eq!(1, g.user.cards.hand(PlayerName::User).len());
    level_up_room(&mut g, 1);
    assert_eq!(3, g.user.cards.hand(PlayerName::User).len());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    spend_actions_until_turn_over(&mut g, Side::Champion);
    assert_eq!(6, g.user.cards.hand(PlayerName::User).len());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    spend_actions_until_turn_over(&mut g, Side::Champion);
    assert_eq!(10, g.user.cards.hand(PlayerName::User).len());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    g.perform(Action::DrawCard(DrawCardAction {}), g.user_id());
    assert_eq!(9, g.user.cards.hand(PlayerName::User).len());
}
