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

use data::game::GameState;
use data::primitives::Side;
use ordered_float::NotNan;
use rules::mana;
use rules::mana::ManaPurpose;

use crate::core::types::notnan;

pub type GameEvaluator = fn(&GameState, side: Side) -> NotNan<f64>;

#[derive(Clone, Debug, Default)]
pub struct HeuristicWeights {
    /// How many more points you have than your opponent
    pub points_difference: NotNan<f64>,
    /// How much more mana you have than your opponent
    pub mana_difference: NotNan<f64>,
    /// How many cards you have in hand
    pub cards_in_hand: NotNan<f64>,
    /// How many cards you have in play
    pub cards_in_play: NotNan<f64>,
    /// How many level counters are on your cards
    pub overlord_level_counters: NotNan<f64>,
}

pub fn standard_evaluator(game: &GameState, side: Side) -> NotNan<f64> {
    run(
        game,
        side,
        HeuristicWeights {
            points_difference: notnan(1_000_000.0),
            mana_difference: notnan(10.0),
            cards_in_hand: notnan(5.0),
            cards_in_play: notnan(15.0),
            overlord_level_counters: notnan(20.0),
        },
    )
}

pub fn run(game: &GameState, side: Side, weights: HeuristicWeights) -> NotNan<f64> {
    let mut result = notnan(0.0);
    result += weights.points_difference
        * (f64::from(game.player(side).score) - f64::from(game.player(side.opponent()).score));
    result += weights.mana_difference
        * (f64::from(mana::get(game, side, ManaPurpose::AllSources))
            - f64::from(mana::get(game, side.opponent(), ManaPurpose::AllSources)));
    if weights.cards_in_hand != 0.0 {
        result += weights.cards_in_hand * game.hand(side).count() as f64;
    }
    if weights.cards_in_play != 0.0 {
        result += weights.cards_in_play
            * game.cards(side).iter().filter(|c| c.position().in_play()).count() as f64;
    }
    if weights.overlord_level_counters != 0.0 && side == Side::Overlord {
        result += weights.overlord_level_counters
            * f64::from(
                game.cards(side)
                    .iter()
                    .filter(|c| c.position().in_play())
                    .map(|c| c.data.card_level)
                    .sum::<u32>(),
            );
    }
    result
}
