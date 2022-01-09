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

//! Functions to query boolean game information, typically whether some game
//! action can currently be taken

use data::card_state::CardPosition;
use data::delegates::{
    CanInitiateRaidQuery, CanLevelUpRoomQuery, CanPlayCardQuery, CanTakeDrawCardActionQuery,
    CanTakeGainManaActionQuery, Flag,
};
use data::game::GameState;
use data::primitives::{CardId, CardType, Side};
use data::prompt::{RaidActivateRoom, RaidAdvance, RaidEncounter};

use crate::{dispatch, queries};

/// Returns whether a given card can currently be played via the basic game
/// action to play a card.
pub fn can_take_play_card_action(game: &GameState, side: Side, card_id: CardId) -> bool {
    let mut can_play = queries::in_main_phase(game, side)
        && side == card_id.side
        && game.card(card_id).position == CardPosition::Hand(side);
    if enters_play_revealed(game, card_id) {
        can_play &= matches!(queries::mana_cost(game, card_id), Some(cost)
                             if cost <= game.player(side).mana);
    }

    dispatch::perform_query(game, CanPlayCardQuery(card_id), Flag::new(can_play)).into()
}

/// Returns true if the indicated card should enter play in the revealed state
/// and is expected to pay its mana cost immediately.
pub fn enters_play_revealed(game: &GameState, card_id: CardId) -> bool {
    matches!(
        crate::get(game.card(card_id).name).card_type,
        CardType::Spell | CardType::Weapon | CardType::Artifact | CardType::Identity
    )
}

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

/// Returns whether the indicated player can currently take the basic game
/// action to initiate a raid.
pub fn can_initiate_raid(game: &GameState, side: Side) -> bool {
    let can_initiate =
        side == Side::Champion && game.data.raid.is_none() && queries::in_main_phase(game, side);
    dispatch::perform_query(game, CanInitiateRaidQuery(side), Flag::new(can_initiate)).into()
}

/// Returns whether the indicated player can currently take the basic game
/// action to level up a room
pub fn can_level_up_room(game: &GameState, side: Side) -> bool {
    let can_level_up =
        side == Side::Overlord && game.player(side).mana > 0 && queries::in_main_phase(game, side);
    dispatch::perform_query(game, CanLevelUpRoomQuery(side), Flag::new(can_level_up)).into()
}

pub fn can_take_raid_activate_room_action(
    _game: &GameState,
    _side: Side,
    _data: RaidActivateRoom,
) -> bool {
    true
}

pub fn can_take_raid_encounter_action(
    _game: &GameState,
    _side: Side,
    _data: RaidEncounter,
) -> bool {
    true
}

pub fn can_take_raid_advance_action(_game: &GameState, _side: Side, _data: RaidAdvance) -> bool {
    true
}

pub fn can_take_raid_destroy_card_action(_game: &GameState, _side: Side, _card_id: CardId) -> bool {
    true
}

pub fn can_take_raid_score_card_action(_game: &GameState, _side: Side, _card_id: CardId) -> bool {
    true
}

pub fn can_take_raid_end_action(_game: &GameState, _side: Side) -> bool {
    true
}
