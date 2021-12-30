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
use protos::spelldawn::{DrawCardAction, PlayerName};
use test_utils::*;

#[test]
fn draw_card() {
    let mut g = new_game(
        Side::Overlord,
        Args { actions: 3, next_draw: Some(CardName::IceDragon), ..Args::default() },
    );
    g.perform_action(Action::DrawCard(DrawCardAction {}));
    assert_identical(vec![CardName::IceDragon], g.hand(PlayerName::User));
}
