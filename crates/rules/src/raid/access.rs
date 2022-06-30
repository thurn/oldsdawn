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

use std::iter;

use anyhow::Result;
use data::card_state::CardPosition;
use data::delegates::{
    CardAccessEvent, ChampionScoreCardEvent, RaidAccessStartEvent, RaidOutcome, ScoreCard,
    ScoreCardEvent,
};
use data::game::{GameState, InternalRaidPhase};
use data::game_actions::{AccessPhaseAction, PromptAction};
use data::primitives::{CardId, CardType, RoomId, Side};
use data::updates::GameUpdate;
use data::with_error::WithError;
use data::{fail, random};

use crate::raid::traits::{RaidDisplayState, RaidPhaseImpl};
use crate::{dispatch, mutations, queries};

/// Final step of a raid, in which cards are accessed by the Champion
#[derive(Debug, Clone, Copy)]
pub struct AccessPhase {}

impl RaidPhaseImpl for AccessPhase {
    type Action = AccessPhaseAction;

    fn unwrap(action: PromptAction) -> Result<AccessPhaseAction> {
        match action {
            PromptAction::AccessPhaseAction(action) => Ok(action),
            _ => fail!("Expected AccessPhaseAction"),
        }
    }

    fn wrap(action: AccessPhaseAction) -> Result<PromptAction> {
        Ok(PromptAction::AccessPhaseAction(action))
    }

    fn enter(self, game: &mut GameState) -> Result<Option<InternalRaidPhase>> {
        dispatch::invoke_event(game, RaidAccessStartEvent(game.raid()?.raid_id))?;
        if game.data.raid.is_none() {
            return Ok(None);
        }

        let accessed = accessed_cards(game)?;
        game.raid_mut()?.accessed = accessed.clone();

        for card_id in &accessed {
            dispatch::invoke_event(game, CardAccessEvent(*card_id))?;
        }

        Ok(None)
    }

    fn actions(self, game: &GameState) -> Result<Vec<AccessPhaseAction>> {
        Ok(game
            .raid()?
            .accessed
            .iter()
            .filter_map(|card_id| access_action_for_card(game, *card_id))
            .chain(iter::once(AccessPhaseAction::EndRaid))
            .collect())
    }

    fn handle_action(
        self,
        game: &mut GameState,
        action: AccessPhaseAction,
    ) -> Result<Option<InternalRaidPhase>> {
        match action {
            AccessPhaseAction::ScoreCard(card_id) => handle_score_card(game, card_id),
            AccessPhaseAction::EndRaid => mutations::end_raid(game, RaidOutcome::Success),
        }?;

        Ok(None)
    }

    fn active_side(self) -> Side {
        Side::Champion
    }

    fn display_state(self, _: &GameState) -> Result<RaidDisplayState> {
        Ok(RaidDisplayState::Access)
    }
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

    Ok(accessed)
}

/// Returns an [AccessPhaseAction] for the Champion to access the provided
/// `card_id`, if any action can be taken.
fn access_action_for_card(game: &GameState, card_id: CardId) -> Option<AccessPhaseAction> {
    let definition = crate::card_definition(game, card_id);
    match definition.card_type {
        CardType::Scheme if can_score_card(game, Side::Champion, card_id) => {
            Some(AccessPhaseAction::ScoreCard(card_id))
        }
        _ => None,
    }
}

/// Can the provided player score the `card_id` card when accessed during a
/// raid?
fn can_score_card(game: &GameState, _side: Side, card_id: CardId) -> bool {
    let raid = match &game.data.raid {
        Some(r) => r,
        None => return false,
    };

    raid.accessed.contains(&card_id)
        && crate::card_definition(game, card_id).config.stats.scheme_points.is_some()
}

fn handle_score_card(game: &mut GameState, card_id: CardId) -> Result<()> {
    game.card_mut(card_id).turn_face_up();
    mutations::move_card(game, card_id, CardPosition::Scoring)?;
    game.raid_mut()?.accessed.retain(|c| *c != card_id);

    game.record_update(|| GameUpdate::ScoreCard(Side::Champion, card_id));

    dispatch::invoke_event(game, ChampionScoreCardEvent(card_id))?;
    dispatch::invoke_event(game, ScoreCardEvent(ScoreCard { player: Side::Champion, card_id }))?;

    let scheme_points = crate::card_definition(game, card_id)
        .config
        .stats
        .scheme_points
        .with_error(|| format!("Expected SchemePoints for {:?}", card_id))?;
    mutations::score_points(game, Side::Champion, scheme_points.points)?;

    mutations::move_card(game, card_id, CardPosition::Scored(Side::Champion))?;
    Ok(())
}
