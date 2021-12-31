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

//! Functions to query game flags, typically whether some game action can
//! currently be taken

use data::delegates::{
    CanPlayCardQuery, CanTakeDrawCardActionQuery, CanTakeGainManaActionQuery, Flag,
};
use data::game::GameState;
use data::primitives::{CardId, Side};

use crate::{dispatch, queries};

/// Returns whether the indicated player can currently take the basic game
/// action to draw a card.
pub fn can_take_draw_card_action(game: &GameState, side: Side) -> bool {
    let can_draw = queries::in_main_phase(game, side) && game.deck(side).next().is_some();
    dispatch::perform_query(game, CanTakeDrawCardActionQuery(side), Flag::new(can_draw)).into()
}

/// Returns whether the indicated player can currently take the basic game
/// action to gain one mana.
pub fn can_take_gain_mana_action(game: &GameState, side: Side) -> bool {
    let can_gain_mana = queries::in_main_phase(game, side);
    dispatch::perform_query(game, CanTakeGainManaActionQuery(side), Flag::new(can_gain_mana)).into()
}

/// Returns whether a given card can currently be played
pub fn can_play(game: &GameState, side: Side, card_id: CardId) -> bool {
    let can_play = queries::in_main_phase(game, side)
        && side == card_id.side
        && matches!(queries::mana_cost(game, card_id), Some(cost) if cost <= game.player(side).mana);
    dispatch::perform_query(game, CanPlayCardQuery(card_id), Flag::new(can_play)).into()
}
