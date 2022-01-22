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

use data::actions::{AdvanceAction, EncounterAction};
use data::card_state::CardPosition;
use data::delegates::{
    CanDefeatTargetQuery, CanEncounterTargetQuery, CanInitiateRaidQuery, CanLevelUpRoomQuery,
    CanPlayCardQuery, CanTakeDrawCardActionQuery, CanTakeGainManaActionQuery, CardEncounter, Flag,
};
use data::game::{GameState, RaidData, RaidPhase};
use data::primitives::{CardId, CardType, Faction, Side};

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

/// Whether a room can currently be activated
pub fn can_take_raid_activate_room_action(game: &GameState, side: Side) -> bool {
    side == Side::Overlord
        && matches!(
            game.data.raid,
            Some(RaidData { phase: RaidPhase::Activation, target, .. })
            if game.has_hidden_defenders(target)
        )
}

/// Whether the provided `source` card is able to target the `target` card with
/// an encounter action. Typically used to determine whether a weapon can target
/// a minion, e.g. based on faction.
pub fn can_encounter_target(game: &GameState, source: CardId, target: CardId) -> bool {
    let can_encounter = matches!(
        (
            crate::card_definition(game, source).config.faction,
            crate::card_definition(game, target).config.faction
        ),
        (Some(source_faction), Some(target_faction))
        if source_faction == Faction::Prismatic || source_faction == target_faction
    );

    dispatch::perform_query(
        game,
        CanEncounterTargetQuery(CardEncounter::new(source, target)),
        Flag::new(can_encounter),
    )
    .into()
}

/// Can the `source` card defeat the `target` card in an encounter by dealing
/// enough damage to equal its health (potentially after paying mana & applying
/// boosts), or via some other game mechanism?
///
/// This requires [can_encounter_target] to be true.
pub fn can_defeat_target(game: &GameState, source: CardId, target: CardId) -> bool {
    let can_defeat = can_encounter_target(game, source, target)
        && matches!(
            queries::cost_to_defeat_target(game, source, target),
            Some(cost)
            if cost <= game.player(source.side).mana
        );

    dispatch::perform_query(
        game,
        CanDefeatTargetQuery(CardEncounter::new(source, target)),
        Flag::new(can_defeat),
    )
    .into()
}

pub fn can_take_raid_encounter_action(
    game: &GameState,
    side: Side,
    action: EncounterAction,
) -> bool {
    let raid = match game.data.raid {
        Some(r) => r,
        None => return false,
    };
    let encounter_position = match raid.phase {
        RaidPhase::Encounter(p) => p,
        _ => return false,
    };
    let defenders = game.defender_list(raid.target);
    let can_continue = side == Side::Champion && defenders.len() > encounter_position;

    if let EncounterAction::UseWeaponAbility(source_id, target_id) = action {
        can_continue
            && defenders[encounter_position].id == target_id
            && can_defeat_target(game, source_id, target_id)
    } else {
        can_continue
    }
}

pub fn can_take_raid_advance_action(_game: &GameState, _side: Side, _data: AdvanceAction) -> bool {
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
