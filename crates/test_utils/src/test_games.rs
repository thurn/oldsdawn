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

use protos::spelldawn::CardIdentifier;

use crate::*;

#[derive(Debug)]
pub struct SimpleIds {
    pub scheme_id: CardIdentifier,
    pub minion_id: CardIdentifier,
    pub weapon_id: CardIdentifier,
}

/// Creates an ongoing [TestGame] with the provided `user_side` and `id_basis`
/// with the following properties:
///
/// - It is the Champion's turn and they have 2 action points remaining
/// - The Overlord has a scheme and a minion in play in the [crate::ROOM_ID]
///   room.
/// - The Champion has a weapon in play
/// - Both players have 999 mana available
///
/// Returns the game along with a [SimpleIds] struct containing the IDs of the
/// created cards
pub fn simple_game(
    user_side: Side,
    scheme: CardName,
    minion: CardName,
    weapon: CardName,
) -> (TestGame, SimpleIds) {
    let mut game = new_game(
        user_side,
        Args { turn: Some(Side::Overlord), turn_actions: 2, ..Args::default() },
    );
    let (_, scheme_id) = game.play_from_hand(scheme);
    let (_, minion_id) = game.play_from_hand(minion);
    let (_, weapon_id) = game.play_from_hand(weapon);

    (game, SimpleIds { scheme_id, minion_id, weapon_id })
}
