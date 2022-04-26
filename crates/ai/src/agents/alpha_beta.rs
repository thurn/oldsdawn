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

use std::cmp;

use anyhow::{bail, Result};
use data::agent_definition::AgentName;
use data::game::{GamePhase, GameState};
use data::game_actions::UserAction;
use data::primitives::Side;
use data::with_error::WithError;
use linkme::distributed_slice;
use ordered_float::NotNan;
use rules::{actions, queries};

use crate::core::types::{notnan, StatePredictionIterator};
use crate::core::{legal_actions, AgentPair, AGENTS};
use crate::heuristics::game_state_evaluator::{standard_evaluator, GameEvaluator};

pub fn initialize() {}

#[distributed_slice(AGENTS)]
static AGENT: AgentPair = (AgentName::AlphaBeta, execute);

/// Implementation of fail-hard alpha/beta pruning
/// See <https://en.wikipedia.org/wiki/Alpha-beta_pruning>
pub fn execute(mut states: StatePredictionIterator, side: Side) -> Result<UserAction> {
    run_search(&states.next().with_error(|| "Expected game state")?.state, side, 4)
}

pub fn run_search(game: &GameState, side: Side, depth: u32) -> Result<UserAction> {
    let mut best_score = notnan(-f64::INFINITY);
    let mut best_action = None;
    for action in legal_actions::evaluate(game, side) {
        // I worry about the performance of .clone() here, but so far it's never shown
        // up in profiling as an issue compared to the cost of
        // `handle_user_action`.
        let mut child = game.clone();
        actions::handle_user_action(&mut child, side, action)?;
        let score = alpha_beta(
            &child,
            side,
            standard_evaluator,
            depth,
            notnan(-f64::INFINITY),
            notnan(f64::INFINITY),
        )?;
        if score > best_score {
            best_score = score;
            best_action = Some(action);
        }
    }

    best_action.with_error(|| "Expected at least one legal action")
}

fn alpha_beta(
    game: &GameState,
    maximizing_side: Side,
    evaluator: GameEvaluator,
    depth: u32,
    mut alpha: NotNan<f64>,
    mut beta: NotNan<f64>,
) -> Result<NotNan<f64>> {
    if depth == 0 || matches!(game.data.phase, GamePhase::GameOver(_)) {
        return Ok(evaluator(game, maximizing_side));
    }
    let current_side = current_priority(game)?;

    if maximizing_side == current_side {
        let mut value = notnan(-f64::INFINITY);
        for action in legal_actions::evaluate(game, current_side) {
            let mut child = game.clone();
            actions::handle_user_action(&mut child, current_side, action)?;
            value = cmp::max(
                value,
                alpha_beta(&child, maximizing_side, evaluator, depth - 1, alpha, beta)?,
            );
            if value >= beta {
                break; // β cutoff
            }
            alpha = alpha.max(value);
        }
        Ok(value)
    } else {
        let mut value = notnan(f64::INFINITY);
        for action in legal_actions::evaluate(game, current_side) {
            let mut child = game.clone();
            actions::handle_user_action(&mut child, current_side, action)?;
            value = cmp::min(
                value,
                alpha_beta(&child, maximizing_side, evaluator, depth - 1, alpha, beta)?,
            );
            if value <= alpha {
                break; // α cutoff
            }
            beta = beta.min(value);
        }
        Ok(value)
    }
}

fn current_priority(game: &GameState) -> Result<Side> {
    if queries::can_take_action(game, Side::Overlord) {
        Ok(Side::Overlord)
    } else if queries::can_take_action(game, Side::Champion) {
        Ok(Side::Champion)
    } else {
        bail!("No player can take action!");
    }
}
