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

use anyhow::{ensure, Result};
use data::actions::{
    ActivateRoomAction, AdvanceAction, EncounterAction, Prompt, PromptAction, PromptContext,
};
use data::card_state::CardPosition;
use data::game::{GameState, RaidData, RaidPhase};
use data::primitives::{CardId, CardType, RaidId, RoomId, Side};
use data::with_error::WithError;
use if_chain::if_chain;
use rand::seq::IteratorRandom;
use rand::thread_rng;

use crate::{flags, mutations, queries};

/// Creates a new raid, setting the [RaidData] for the provided `game` to
/// reference the provided `target` room and `RaidPhase`.
///
/// Does not invoke 'raid start' or similar callbacks. Returns the [RaidId] for
/// the newly-created raid.
pub fn create_raid(game: &mut GameState, target: RoomId, phase: RaidPhase) -> Result<RaidId> {
    ensure!(game.data.raid.is_none(), "Game already has an active raid!");
    let raid_id = RaidId(game.data.next_raid_id);
    let raid = RaidData { target, raid_id, phase, room_active: false, accessed: vec![] };
    game.data.next_raid_id += 1;
    game.data.raid = Some(raid);
    on_enter_raid_phase(game)?;
    Ok(raid_id)
}

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
        RaidPhase::Activation => {}
        RaidPhase::Encounter(defender_index) => {
            let defender_id = find_defender(game, game.raid()?.target, defender_index)?;
            if_chain! {
                if game.raid()?.room_active;
                if !game.card(defender_id).data.revealed_to_opponent;
                if let Some(cost) = queries::mana_cost(game, defender_id);
                if cost <= game.player(Side::Overlord).mana;
                then {
                    // Pay for and reveal defender
                    mutations::spend_mana(game, Side::Overlord, cost);
                    mutations::set_revealed(game, defender_id, true);
                }
            }
        }
        RaidPhase::Continue(_) => {
            todo!()
        }
        RaidPhase::Access => {
            game.raid_mut()?.accessed = accessed_cards(game)?;
        }
    }

    set_raid_prompt(game)
}

/// Returns a vector of the cards accessed for the current raid target, mutating
/// the [GameState] to store the results of random zone selections.
fn accessed_cards(game: &mut GameState) -> Result<Vec<CardId>> {
    let target = game.raid()?.target;

    let accessed = match target {
        RoomId::Vault => {
            mutations::top_of_deck(game, Side::Overlord, queries::vault_access_count(game))
        }
        RoomId::Sanctum => {
            let count = queries::sanctum_access_count(game);
            if game.data.config.deterministic {
                game.hand(Side::Overlord).map(|c| c.id).take(count).collect()
            } else {
                game.hand(Side::Overlord).map(|c| c.id).choose_multiple(&mut thread_rng(), count)
            }
        }
        RoomId::Crypts => game
            .card_list_for_position(Side::Overlord, CardPosition::DiscardPile(Side::Overlord))
            .iter()
            .filter(|c| !c.data.revealed_to_opponent)
            .map(|c| c.id)
            .collect(),
        _ => game.occupants(target).map(|c| c.id).collect(),
    };

    for card_id in &accessed {
        mutations::set_revealed(game, *card_id, true);
    }

    Ok(accessed)
}

/// Sets a UI [Prompt] for the current raid state of the provided `game`.
///
/// Only one player at a time receives a prompt, while their opponent sees a
/// 'waiting' indicator.
pub fn set_raid_prompt(game: &mut GameState) -> Result<()> {
    let (active_player, prompt) = match game.raid()?.phase {
        RaidPhase::Activation => (Side::Overlord, build_activation_prompt()),
        RaidPhase::Encounter(defender) => (Side::Champion, build_encounter_prompt(game, defender)?),
        RaidPhase::Continue(_) => (Side::Champion, build_continue_prompt()),
        RaidPhase::Access => (Side::Champion, build_access_prompt(game)?),
    };

    mutations::set_prompt(game, active_player, prompt);
    Ok(())
}

fn build_activation_prompt() -> Prompt {
    Prompt {
        context: Some(PromptContext::ActivateRoom),
        responses: vec![
            PromptAction::ActivateRoomAction(ActivateRoomAction::Activate),
            PromptAction::ActivateRoomAction(ActivateRoomAction::Pass),
        ],
    }
}

fn build_encounter_prompt(game: &GameState, defender: usize) -> Result<Prompt> {
    let defender_id = find_defender(game, game.raid()?.target, defender)?;
    Ok(Prompt {
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
            .chain(iter::once(PromptAction::EncounterAction(EncounterAction::Continue)))
            .collect(),
    })
}

fn build_continue_prompt() -> Prompt {
    Prompt {
        context: Some(PromptContext::RaidAdvance),
        responses: vec![
            PromptAction::AdvanceAction(AdvanceAction::Advance),
            PromptAction::AdvanceAction(AdvanceAction::Retreat),
        ],
    }
}

fn build_access_prompt(game: &GameState) -> Result<Prompt> {
    Ok(Prompt {
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
        CardType::Scheme if flags::can_score_card(game, Side::Champion, card_id) => {
            Some(PromptAction::RaidScoreCard(card_id))
        }
        CardType::Project | CardType::Upgrade
            if flags::can_destroy_accessed_card(game, card_id) =>
        {
            Some(PromptAction::RaidDestroyCard(card_id))
        }
        _ => None,
    }
}

/// Finds the defending [CardId] at the given `index` position in the indicated
/// `room_id`.
pub fn find_defender(game: &GameState, room_id: RoomId, index: usize) -> Result<CardId> {
    Ok(game.defender_list(room_id).get(index).with_error(|| "Defender Not Found")?.id)
}
