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

//! Updates for the current phase of a raid.

use std::iter;

use anyhow::Result;
use data::card_state::CardPosition;
use data::delegates::{CardAccessEvent, EncounterMinionEvent, RaidAccessStartEvent};
use data::game::{GameState, RaidPhase};
use data::game_actions::{
    ContinueAction, EncounterAction, GamePrompt, PromptAction, PromptContext, RoomActivationAction,
};
use data::primitives::{CardId, CardType, RoomId, Side};
use data::random;
use data::updates::GameUpdate;
use data::with_error::WithError;

use crate::mana::ManaPurpose;
use crate::mutations::{SetPrompt, SummonMinion};
use crate::{dispatch, flags, mana, mutations, queries};

/// Updates the [RaidPhase] for the ongoing raid in the provided `game` and
/// invokes callbacks as appropriate.
pub fn set_raid_phase(game: &mut GameState, phase: RaidPhase) -> Result<()> {
    game.raid_mut()?.phase = phase;
    on_enter_raid_phase(game)
}

/// Function to apply updates for the current [RaidPhase] of the provided
/// `game`.
fn on_enter_raid_phase(game: &mut GameState) -> Result<()> {
    match game.raid()?.phase {
        RaidPhase::Begin => {}
        RaidPhase::Activation => {}
        RaidPhase::Encounter(defender_index) => {
            let defender_id = find_defender(game, game.raid()?.target, defender_index)?;
            if can_summon_defender(game, defender_index)? {
                mutations::summon_minion(game, defender_id, SummonMinion::PayCosts)?;
            }
            dispatch::invoke_event(game, EncounterMinionEvent(defender_id))?;
        }
        RaidPhase::Continue(_) => {}
        RaidPhase::Access => {
            dispatch::invoke_event(game, RaidAccessStartEvent(game.raid()?.raid_id))?;
            let accessed = accessed_cards(game)?;

            for card_id in &accessed {
                dispatch::invoke_event(game, CardAccessEvent(*card_id))?;
            }
            game.raid_mut()?.accessed = accessed;
        }
    }

    set_raid_prompt(game)
}

/// Returns true if the raid defender at `defender_index` is currently face down
/// and could be turned face up automatically by paying its mana cost.
///
/// Returns an error if there is no active raid or if this is an invalid
/// defender index.
pub fn can_summon_defender(game: &GameState, defender_index: usize) -> Result<bool> {
    let raid = game.raid()?;
    let defender_id = find_defender(game, raid.target, defender_index)?;
    let mut can_summon = raid.room_active && game.card(defender_id).is_face_down();

    if let Some(cost) = queries::mana_cost(game, defender_id) {
        can_summon &= cost <= mana::get(game, Side::Overlord, ManaPurpose::PayForCard(defender_id))
    }

    if let Some(custom_cost) = &crate::card_definition(game, defender_id).cost.custom_cost {
        can_summon &= (custom_cost.can_pay)(game, defender_id);
    }

    Ok(can_summon)
}

/// Returns a vector of the cards accessed for the current raid target, mutating
/// the [GameState] to store the results of random zone selections and mark
/// cards as revealed.
fn accessed_cards(game: &mut GameState) -> Result<Vec<CardId>> {
    let target = game.raid()?.target;

    let accessed = match target {
        RoomId::Vault => mutations::realize_top_of_deck(
            game,
            Side::Overlord,
            queries::vault_access_count(game)?,
        )?,
        RoomId::Sanctum => {
            let count = queries::sanctum_access_count(game)?;
            random::cards_in_position(
                game,
                Side::Overlord,
                CardPosition::Hand(Side::Overlord),
                count as usize,
            )
        }
        RoomId::Crypts => {
            game.card_list_for_position(Side::Overlord, CardPosition::DiscardPile(Side::Overlord))
        }
        _ => game.occupants(target).map(|c| c.id).collect(),
    };

    for card_id in &accessed {
        game.card_mut(*card_id).set_revealed_to(Side::Champion, true);
    }

    game.push_update(|| GameUpdate::CardsAccessed(accessed.clone()));

    Ok(accessed)
}

/// Sets a UI [GamePrompt] for the current raid state of the provided `game`.
///
/// Only one player at a time receives a prompt, while their opponent sees a
/// 'waiting' indicator.
pub fn set_raid_prompt(game: &mut GameState) -> Result<()> {
    let (active_player, prompt) = match game.raid()?.phase {
        RaidPhase::Begin => return Ok(()),
        RaidPhase::Activation => (Side::Overlord, build_activation_prompt()),
        RaidPhase::Encounter(defender) => (Side::Champion, build_encounter_prompt(game, defender)?),
        RaidPhase::Continue(_) => (Side::Champion, build_continue_prompt()),
        RaidPhase::Access => (Side::Champion, build_access_prompt(game)?),
    };

    mutations::set_prompt(game, active_player, SetPrompt::GamePrompt, prompt)?;
    Ok(())
}

fn build_activation_prompt() -> GamePrompt {
    GamePrompt {
        context: Some(PromptContext::ActivateRoom),
        responses: vec![
            PromptAction::ActivateRoomAction(RoomActivationAction::Activate),
            PromptAction::ActivateRoomAction(RoomActivationAction::Pass),
        ],
    }
}

fn build_encounter_prompt(game: &GameState, defender: usize) -> Result<GamePrompt> {
    let defender_id = find_defender(game, game.raid()?.target, defender)?;
    Ok(GamePrompt {
        context: None,
        responses: game
            .weapons()
            .filter(|weapon| flags::can_defeat_target(game, weapon.id, defender_id))
            .map(|weapon| {
                PromptAction::EncounterAction(EncounterAction::UseWeaponAbility(
                    weapon.id,
                    defender_id,
                ))
            })
            .chain(queries::minion_combat_actions(game, defender_id))
            .collect(),
    })
}

fn build_continue_prompt() -> GamePrompt {
    GamePrompt {
        context: Some(PromptContext::RaidAdvance),
        responses: vec![
            PromptAction::ContinueAction(ContinueAction::Advance),
            PromptAction::ContinueAction(ContinueAction::Retreat),
        ],
    }
}

fn build_access_prompt(game: &GameState) -> Result<GamePrompt> {
    Ok(GamePrompt {
        context: None,
        responses: game
            .raid()?
            .accessed
            .iter()
            .filter_map(|card_id| access_prompt_for_card(game, *card_id))
            .chain(iter::once(PromptAction::EndRaid))
            .collect(),
    })
}

/// Returns a [PromptAction] for the Champion to access the provided `card_id`,
/// if any action can be taken.
fn access_prompt_for_card(game: &GameState, card_id: CardId) -> Option<PromptAction> {
    let definition = crate::card_definition(game, card_id);
    match definition.card_type {
        CardType::Scheme if flags::can_score_card_when_accessed(game, Side::Champion, card_id) => {
            Some(PromptAction::RaidScoreCard(card_id))
        }
        CardType::Project if flags::can_destroy_accessed_card(game, card_id) => {
            Some(PromptAction::RaidDestroyCard(card_id))
        }
        _ => None,
    }
}

/// Finds the defending [CardId] at the given `index` position in the indicated
/// `room_id`.
pub fn find_defender(game: &GameState, room_id: RoomId, index: usize) -> Result<CardId> {
    Ok(*game.defender_list(room_id).get(index).with_error(|| "Defender Not Found")?)
}
