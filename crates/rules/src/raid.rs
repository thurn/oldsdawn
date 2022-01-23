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

//! Handling for raid-related user actions.

use std::iter;

use anyhow::{bail, ensure, Context, Result};
use data::actions::{
    ActivateRoomAction, AdvanceAction, EncounterAction, Prompt, PromptAction, PromptContext,
};
use data::game::{GameState, RaidPhase};
use data::primitives::{CardId, CardType, RoomId, Side};
use data::updates::{GameUpdate, InteractionObjectId, TargetedInteraction};
use if_chain::if_chain;
use tracing::{info, instrument};

use crate::{flags, mutations, queries};

#[instrument(skip(game))]
pub fn initiate_raid_action(
    game: &mut GameState,
    user_side: Side,
    target_room: RoomId,
) -> Result<()> {
    info!(?user_side, "initiate_raid_action");
    ensure!(flags::can_initiate_raid(game, user_side), "Cannot initiate raid for {:?}", user_side);
    mutations::spend_action_points(game, user_side, 1);
    mutations::initiate_raid(game, target_room);
    Ok(())
}

#[instrument(skip(game))]
pub fn activate_room_action(
    game: &mut GameState,
    user_side: Side,
    data: ActivateRoomAction,
) -> Result<()> {
    info!(?user_side, ?data, "raid_activate_room_action");
    ensure!(
        flags::can_take_raid_activate_room_action(game, user_side),
        "Cannot activate room for {:?}",
        user_side
    );

    let defender_count = game
        .defenders_alphabetical(game.data.raid.with_context(|| "No active raid")?.target)
        .count();
    game.raid_mut()?.active = data == ActivateRoomAction::Activate;

    if defender_count == 0 {
        return initiate_access_phase(game);
    }

    game.raid_mut()?.phase = RaidPhase::Encounter(defender_count - 1);
    let target = game.raid()?.target;
    let defender_id =
        game.defender_list(target).get(defender_count - 1).with_context(|| "No defender")?.id;

    if_chain! {
        if let Some(cost) = queries::mana_cost(game, defender_id);
        if cost <= game.player(Side::Overlord).mana;
        then {
            mutations::spend_mana(game, Side::Overlord, cost);
            mutations::set_revealed(game, defender_id, true);

            mutations::set_prompt(
                game,
                Side::Champion,
                Prompt {
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
                        .chain(iter::once(PromptAction::EncounterAction(
                            EncounterAction::Continue,
                        )))
                        .collect(),
                },
            );
        } else {
            todo!("Continue")
        }
    }

    Ok(())
}

#[instrument(skip(game))]
pub fn encounter_action(
    game: &mut GameState,
    user_side: Side,
    action: EncounterAction,
) -> Result<()> {
    info!(?user_side, ?action, "raid_encounter_action");
    ensure!(
        flags::can_take_raid_encounter_action(game, user_side, action),
        "Cannot take encounter action for {:?}",
        user_side
    );

    let encounter_number = match game.data.raid.with_context(|| "Expected Raid")?.phase {
        RaidPhase::Encounter(n) => n,
        _ => bail!("Expected Encounter phase"),
    };

    match action {
        EncounterAction::UseWeaponAbility(source_id, target_id) => {
            let cost =
                queries::cost_to_defeat_target(game, source_id, target_id).with_context(|| {
                    format!("{:?} cannot defeat target: {:?}", source_id, target_id)
                })?;
            mutations::spend_mana(game, user_side, cost);
            game.updates.push(GameUpdate::TargetedInteraction(TargetedInteraction {
                source: InteractionObjectId::CardId(source_id),
                target: InteractionObjectId::CardId(target_id),
            }))
        }
        EncounterAction::Continue => {
            todo!("Fire weapon effects")
        }
    }

    if encounter_number == 0 {
        initiate_access_phase(game)?;
    } else {
        game.raid_mut()?.phase = RaidPhase::Continue(encounter_number - 1);
        mutations::set_prompt(
            game,
            user_side,
            Prompt {
                context: Some(PromptContext::RaidAdvance),
                responses: vec![
                    PromptAction::AdvanceAction(AdvanceAction::Advance),
                    PromptAction::AdvanceAction(AdvanceAction::Retreat),
                ],
            },
        );
    }

    Ok(())
}

#[instrument(skip(game))]
pub fn advance_action(game: &mut GameState, user_side: Side, data: AdvanceAction) -> Result<()> {
    info!(?user_side, ?data, "raid_advance_action");
    ensure!(
        flags::can_take_raid_advance_action(game, user_side, data),
        "Cannot take advance action for {:?}",
        user_side
    );
    Ok(())
}

#[instrument(skip(game))]
pub fn destroy_card_action(game: &mut GameState, user_side: Side, card_id: CardId) -> Result<()> {
    info!(?user_side, ?card_id, "raid_destroy_card_action");
    ensure!(
        flags::can_take_raid_destroy_card_action(game, user_side, card_id),
        "Cannot take destroy card action for {:?}",
        user_side
    );
    Ok(())
}

#[instrument(skip(game))]
pub fn score_card_action(game: &mut GameState, user_side: Side, card_id: CardId) -> Result<()> {
    info!(?user_side, ?card_id, "raid_score_card_action");
    ensure!(
        flags::can_take_raid_score_card_action(game, user_side, card_id),
        "Cannot take score card action for {:?}",
        user_side
    );
    Ok(())
}

#[instrument(skip(game))]
pub fn raid_end_action(game: &mut GameState, user_side: Side) -> Result<()> {
    info!(?user_side, "raid_end_action");
    ensure!(
        flags::can_take_raid_end_action(game, user_side),
        "Cannot take raid end action for {:?}",
        user_side
    );
    Ok(())
}

/// Invoked once all of the defenders for a room during a raid (if any) have
/// been passed.
fn initiate_access_phase(game: &mut GameState) -> Result<()> {
    game.raid_mut()?.phase = RaidPhase::Access;
    let target = game.raid()?.target;

    match target {
        RoomId::Vault => {
            todo!("Access Vault")
        }
        RoomId::Sanctum => {
            todo!("Access Sanctum")
        }
        RoomId::Crypts => {
            todo!("Access Crypts")
        }
        _ => {
            let occupants = game.occupants(target).map(|c| c.id).collect::<Vec<_>>();
            for occupant_id in occupants {
                mutations::set_revealed(game, occupant_id, true);
            }

            mutations::set_prompt(
                game,
                Side::Champion,
                Prompt {
                    context: None,
                    responses: game
                        .occupants(target)
                        .filter_map(|c| access_prompt_for_card(game, c.id))
                        .chain(iter::once(PromptAction::EndRaid))
                        .collect(),
                },
            )
        }
    }

    Ok(())
}

/// Returns a [PromptAction] for the Champion to access the provided `card_id`,
/// if any action can be taken.
fn access_prompt_for_card(game: &GameState, card_id: CardId) -> Option<PromptAction> {
    let definition = crate::card_definition(game, card_id);
    match definition.card_type {
        CardType::Scheme => Some(PromptAction::RaidScoreCard(card_id)),
        CardType::Project | CardType::Upgrade
            if flags::can_destroy_accessed_card(game, card_id) =>
        {
            Some(PromptAction::RaidDestroyCard(card_id))
        }
        _ => None,
    }
}
