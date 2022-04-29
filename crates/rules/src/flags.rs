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

use data::card_definition::{AbilityType, CustomTargeting};
use data::card_state::{CardPosition, CardState};
use data::delegates::{
    CanActivateAbilityQuery, CanDefeatTargetQuery, CanEncounterTargetQuery, CanInitiateRaidQuery,
    CanLevelUpRoomQuery, CanPlayCardQuery, CanTakeDrawCardActionQuery, CanTakeGainManaActionQuery,
    CardEncounter, Flag,
};
use data::game::{GamePhase, GameState, RaidData, RaidPhase, RaidPhaseKind};
use data::game_actions::{CardTarget, EncounterAction};
use data::primitives::{AbilityId, CardId, CardType, Faction, RoomId, Side};

use crate::{dispatch, queries};

/// Returns whether a player can currently make a mulligan decision
pub fn can_make_mulligan_decision(game: &GameState, side: Side) -> bool {
    matches!(
        &game.data.phase, GamePhase::ResolveMulligans(mulligan) if mulligan.decision(side).is_none()
    )
}

/// Returns whether a given card can currently be played via the basic game
/// action to play a card.
pub fn can_take_play_card_action(
    game: &GameState,
    side: Side,
    card_id: CardId,
    target: CardTarget,
) -> bool {
    let mut can_play = queries::in_main_phase(game, side)
        && side == card_id.side
        && game.card(card_id).position() == CardPosition::Hand(side)
        && is_valid_target(game, card_id, target);
    // TODO: Check action cost

    if enters_play_face_up(game, card_id) {
        can_play &= matches!(queries::mana_cost(game, card_id), Some(cost)
                             if cost <= game.player(side).mana);
    }

    dispatch::perform_query(game, CanPlayCardQuery(card_id), Flag::new(can_play)).into()
}

pub fn can_take_activate_ability_action(
    game: &GameState,
    side: Side,
    ability_id: AbilityId,
) -> bool {
    let card = game.card(ability_id.card_id);
    let cost = match &crate::get(card.name)
        .abilities
        .get(ability_id.index.value())
        .map(|a| &a.ability_type)
    {
        Some(AbilityType::Activated(cost)) => cost,
        _ => return false,
    };

    let mut can_activate = queries::in_main_phase(game, side)
        && side == ability_id.card_id.side
        && card.position().in_play()
        && cost.actions <= game.player(side).actions;

    if let Some(cost) = queries::ability_mana_cost(game, ability_id) {
        can_activate &= cost <= game.player(side).mana;
    }

    dispatch::perform_query(game, CanActivateAbilityQuery(ability_id), Flag::new(can_activate))
        .into()
}

fn is_valid_target(game: &GameState, card_id: CardId, target: CardTarget) -> bool {
    fn room_can_add(game: &GameState, room_id: RoomId, card_types: Vec<CardType>) -> bool {
        !room_id.is_inner_room()
            && !game
                .occupants(room_id)
                .any(|card| card_types.contains(&crate::get(card.name).card_type))
    }

    let definition = crate::get(game.card(card_id).name);
    if let Some(targeting) = &definition.config.custom_targeting {
        let room_id = match target {
            CardTarget::Room(room_id) => room_id,
            _ => return false,
        };

        return match targeting {
            CustomTargeting::TargetRoom(predicate) => predicate(room_id),
        };
    }

    match definition.card_type {
        CardType::Spell | CardType::Weapon | CardType::Artifact | CardType::Sorcery => {
            target == CardTarget::None
        }
        CardType::Minion => matches!(target, CardTarget::Room(_)),
        CardType::Project | CardType::Scheme => {
            matches!(target, CardTarget::Room(room_id)
                if room_can_add(game, room_id, vec![CardType::Project, CardType::Scheme]))
        }
        CardType::Identity => false,
    }
}

/// Returns true if the indicated card should enter play in the face up state
/// and is expected to pay its mana cost immediately.
pub fn enters_play_face_up(game: &GameState, card_id: CardId) -> bool {
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
/// action to initiate a raid on the target [RoomId].
pub fn can_take_initiate_raid_action(game: &GameState, side: Side, target: RoomId) -> bool {
    let non_empty = target.is_inner_room() || game.occupants(target).next().is_some();
    let can_initiate = non_empty
        && side == Side::Champion
        && game.data.raid.is_none()
        && queries::in_main_phase(game, side);
    dispatch::perform_query(game, CanInitiateRaidQuery(side), Flag::new(can_initiate)).into()
}

/// Returns whether the indicated player can currently take the basic game
/// action to level up a room
pub fn can_take_level_up_room_action(game: &GameState, side: Side, room_id: RoomId) -> bool {
    let has_level_card = game
        .occupants(room_id)
        .chain(game.defenders_alphabetical(room_id))
        .any(|card| crate::get(card.name).config.stats.can_level_up);
    let can_level_up = has_level_card
        && side == Side::Overlord
        && game.player(side).mana > 0
        && queries::in_main_phase(game, side);
    dispatch::perform_query(game, CanLevelUpRoomQuery(side), Flag::new(can_level_up)).into()
}

/// Whether a room can currently be activated
pub fn can_take_room_activation_action(game: &GameState, side: Side) -> bool {
    side == Side::Overlord
        && matches!(
            game.data.raid,
            Some(RaidData { phase: RaidPhase::Activation, target, .. })
            if game.defenders_alphabetical(target).any(CardState::is_face_down)
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
    let raid = match &game.data.raid {
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

/// Whether the `side` user can take a raid `ContinueAction`.
pub fn can_take_continue_action(game: &GameState, side: Side) -> bool {
    side == Side::Champion
        && matches!(game.data.raid, Some(ref raid) if raid.phase.kind() == RaidPhaseKind::Continue)
}

/// Can the Champion player destroy the accessed card `card_id`?
pub fn can_destroy_accessed_card(_game: &GameState, _card_id: CardId) -> bool {
    false
}

pub fn can_take_raid_destroy_card_action(_game: &GameState, _side: Side, _card_id: CardId) -> bool {
    true
}

/// Can the provided player score the `card_id` card when accessed during a
/// raid?
pub fn can_score_card_when_accessed(game: &GameState, side: Side, card_id: CardId) -> bool {
    let raid = match &game.data.raid {
        Some(r) => r,
        None => return false,
    };

    side == Side::Champion
        && raid.phase == RaidPhase::Access
        && raid.accessed.contains(&card_id)
        && crate::card_definition(game, card_id).config.stats.scheme_points.is_some()
}

pub fn can_take_raid_end_action(game: &GameState, side: Side) -> bool {
    let raid = match &game.data.raid {
        Some(r) => r,
        None => return false,
    };

    side == Side::Champion && raid.phase == RaidPhase::Access
}
