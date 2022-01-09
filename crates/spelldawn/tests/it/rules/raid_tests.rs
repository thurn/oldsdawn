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

use data::primitives::Side;
use insta::assert_debug_snapshot;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::{InitiateRaidAction, PlayerName};
use test_utils::client::HasText;
use test_utils::{test_games, *};

#[test]
fn initiate_raid() {
    let mut g = test_games::simple_game(Side::Champion, Some(1000));
    let response = g.perform_action(
        Action::InitiateRaid(InitiateRaidAction { room_id: CLIENT_ROOM_ID.into() }),
        g.user_id(),
    );
    assert_eq!(1, g.player().actions());
    assert_eq!(PlayerName::Opponent, g.user.data.priority());
    assert_eq!(PlayerName::User, g.opponent.data.priority());
    assert_commands_match(&response, vec!["UpdateGameView", "RenderInterface", "InitiateRaid"]);
    assert!(g.user.interface.main_controls().has_text("Waiting"));
    assert!(g.opponent.interface.main_controls().has_text("Activate"));
    assert!(g.opponent.interface.main_controls().has_text("Pass"));
    assert_debug_snapshot!(response);
}
