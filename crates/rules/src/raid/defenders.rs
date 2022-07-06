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

use anyhow::Result;
use data::game::GameState;
use data::primitives::{CardId, RoomId, Side};
use data::utils;
use fallible_iterator::FallibleIterator;
use with_error::WithError;

use crate::mana::ManaPurpose;
use crate::{mana, queries};

/// Returns true if the raid `defender_id` is currently face down and could be
/// turned face up automatically by paying its mana cost.
///
/// Returns an error if there is no active raid or if this is an invalid
/// defender.
pub fn can_summon_defender(game: &GameState, defender_id: CardId) -> Result<bool> {
    let raid = game.raid()?;
    let mut can_summon = raid.room_active && game.card(defender_id).is_face_down();

    if let Some(cost) = queries::mana_cost(game, defender_id) {
        can_summon &= cost <= mana::get(game, Side::Overlord, ManaPurpose::PayForCard(defender_id))
    }

    if let Some(custom_cost) = &crate::card_definition(game, defender_id).cost.custom_cost {
        can_summon &= (custom_cost.can_pay)(game, defender_id);
    }

    Ok(can_summon)
}

/// Searches for the next defender to encounter during an ongoing raid with a
/// position less than the provided index (or any index if not provided). If an
/// eligible defender is available with position < `less_than`, its index is
/// returned.
///
/// An 'eligible' defender is either one which is face up, or one which *can* be
/// turned face up by paying its costs.
pub fn next_encounter(game: &GameState, less_than: Option<usize>) -> Result<Option<usize>> {
    let target = game.raid()?.target;
    let defenders = game.defender_list(target);
    let mut reversed = utils::fallible(defenders.iter().enumerate().rev());
    let found = reversed.find(|(index, card_id)| {
        let in_range = less_than.map_or(true, |less_than| *index < less_than);
        let defender_id = find_defender(game, target, *index)?;
        let can_encounter =
            game.card(**card_id).is_face_up() || can_summon_defender(game, defender_id)?;
        Ok(in_range && can_encounter)
    })?;

    Ok(found.map(|(index, _)| index))
}

fn find_defender(game: &GameState, room_id: RoomId, index: usize) -> Result<CardId> {
    Ok(*game.defender_list(room_id).get(index).with_error(|| "Defender Not Found")?)
}
