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
use data::delegates::{
    EncounterMinionEvent, MinionCombatAbilityEvent, MinionCombatActionsQuery, MinionDefeatedEvent,
    UsedWeapon, UsedWeaponEvent,
};
use data::fail;
use data::game::{GameState, RaidState};
use data::game_actions::{EncounterAction, PromptAction};
use data::primitives::{CardId, GameObjectId, Side};
use data::updates::{GameUpdate, TargetedInteraction};
use data::with_error::WithError;

use crate::mana::ManaPurpose;
use crate::mutations::SummonMinion;
use crate::raid::core::RaidStateNode;
use crate::raid::defenders;
use crate::{card_prompt, dispatch, flags, mana, mutations, queries};

#[derive(Debug, Clone, Copy)]
pub struct EncounterState {}

impl RaidStateNode<EncounterAction> for EncounterState {
    fn unwrap(action: PromptAction) -> Result<EncounterAction> {
        match action {
            PromptAction::EncounterAction(action) => Ok(action),
            _ => fail!("Expected EncounterAction"),
        }
    }

    fn wrap(action: EncounterAction) -> Result<PromptAction> {
        Ok(PromptAction::EncounterAction(action))
    }

    fn enter(self, game: &mut GameState) -> Result<Option<RaidState>> {
        if defenders::can_summon_defender(game, game.raid_defender()?)? {
            mutations::summon_minion(game, game.raid_defender()?, SummonMinion::PayCosts)?;
            if game.data.raid.is_none() {
                return Ok(None);
            }
        }

        dispatch::invoke_event(game, EncounterMinionEvent(game.raid_defender()?))?;
        Ok(None)
    }

    fn actions(self, game: &GameState) -> Result<Vec<EncounterAction>> {
        let defender_id = game.raid_defender()?;
        Ok(game
            .weapons()
            .filter(|weapon| flags::can_defeat_target(game, weapon.id, defender_id))
            .map(|weapon| EncounterAction::UseWeaponAbility(weapon.id, defender_id))
            .chain(minion_combat_actions(game, defender_id))
            .collect())
    }

    fn active_side(self) -> Side {
        Side::Champion
    }

    fn handle_action(
        self,
        game: &mut GameState,
        action: EncounterAction,
    ) -> Result<Option<RaidState>> {
        match action {
            EncounterAction::UseWeaponAbility(source_id, target_id) => {
                let cost = queries::cost_to_defeat_target(game, source_id, target_id).with_error(
                    || format!("{:?} cannot defeat target: {:?}", source_id, target_id),
                )?;
                mana::spend(game, Side::Champion, ManaPurpose::UseWeapon(source_id), cost)?;

                game.record_update(|| {
                    GameUpdate::TargetedInteraction(TargetedInteraction {
                        source: GameObjectId::CardId(source_id),
                        target: GameObjectId::CardId(target_id),
                    })
                });

                dispatch::invoke_event(
                    game,
                    UsedWeaponEvent(UsedWeapon {
                        raid_id: game.raid()?.raid_id,
                        weapon_id: source_id,
                        target_id,
                        mana_spent: cost,
                    }),
                )?;
                dispatch::invoke_event(game, MinionDefeatedEvent(target_id))?;
            }
            EncounterAction::NoWeapon | EncounterAction::CardAction(_) => {
                let defender_id = game.raid_defender()?;
                // TODO: This assumes card actions are always negative
                game.record_update(|| {
                    GameUpdate::TargetedInteraction(TargetedInteraction {
                        source: GameObjectId::CardId(defender_id),
                        target: GameObjectId::Identity(Side::Champion),
                    })
                });
                dispatch::invoke_event(game, MinionCombatAbilityEvent(defender_id))?;
            }
        }

        if let EncounterAction::CardAction(card_action) = action {
            card_prompt::handle(game, Side::Champion, card_action)?;
        }

        Ok(if game.data.raid.is_none() {
            // Abilities may have ended the raid
            None
        } else if game.raid_encounter()? > 100 {
            game.raid_mut()?.encounter = Some(game.raid_encounter()? - 999);
            Some(RaidState::Continue)
        } else if let Some(encounter) =
            defenders::next_encounter(game, Some(game.raid_encounter()?))?
        {
            game.raid_mut()?.encounter = Some(encounter);
            Some(RaidState::Continue)
        } else {
            Some(RaidState::Access)
        })
    }
}

/// Actions to present when a minion is encountered in combat in addition to
/// weapon abilities.
fn minion_combat_actions(game: &GameState, minion_id: CardId) -> Vec<EncounterAction> {
    let result = dispatch::perform_query(game, MinionCombatActionsQuery(minion_id), vec![])
        .into_iter()
        .flatten()
        .map(EncounterAction::CardAction)
        .collect::<Vec<_>>();
    if result.is_empty() {
        vec![EncounterAction::NoWeapon]
    } else {
        result
    }
}
