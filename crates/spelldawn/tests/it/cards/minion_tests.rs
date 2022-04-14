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
use test_utils::*;

#[test]
fn ice_dragon() {
    let mut g = new_game(Side::Overlord, Args { opponent_hand_size: 5, ..Args::default() });
    g.play_from_hand(CardName::IceDragon);
    fire_minion_combat_abilities(&mut g);
    assert!(!g.user.data.raid_active());
    assert_eq!(1, g.user.cards.discard_pile(PlayerName::Opponent).len());
    assert_eq!(5, g.user.cards.hand(PlayerName::Opponent).len()); // Card is drawn for turn!
}
