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

use std::time::Duration;

use ai::core::legal_actions;
use ai::tournament::run_tournament;
use ai::tournament::run_tournament::RunGames;
use ai_core::agent;
use ai_core::agent::{Agent, AgentData};
use ai_monte_carlo::monte_carlo::{MonteCarloAlgorithm, RandomPlayoutEvaluator};
use ai_monte_carlo::uct1::Uct1;
use ai_testing::nim::{NimState, NimWinLossEvaluator};
use ai_tree_search::alpha_beta::AlphaBetaAlgorithm;
use ai_tree_search::minimax::MinimaxAlgorithm;
use cards::{decklists, initialize};
use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, Criterion};
use data::agent_definition::AgentName;
use data::primitives::Side;

criterion_group!(benches, legal_actions, random_actions, minimax_nim, alpha_beta_nim, uct1_nim);
criterion_main!(benches);

fn configure(group: &mut BenchmarkGroup<WallTime>) {
    initialize::run();
    group.confidence_level(0.99).noise_threshold(0.025).measurement_time(Duration::from_secs(60));
}

pub fn legal_actions(c: &mut Criterion) {
    let mut group = c.benchmark_group("legal_actions");
    configure(&mut group);
    let game = decklists::canonical_game().unwrap();
    group.bench_function("legal_actions", |b| {
        b.iter(|| {
            let _actions =
                legal_actions::evaluate(&game, Side::Overlord).unwrap().collect::<Vec<_>>();
        })
    });
    group.finish();
}

pub fn random_actions(c: &mut Criterion) {
    // NOTE: Keep in mind that if the behavior of legal_actions() changes, you
    // can't meaningfully compare benchmark results before and after. The games will
    // take a completely different path, meaning the games might end more quickly
    // even if all the code is slower.
    let mut group = c.benchmark_group("random_actions");
    configure(&mut group);
    group.bench_function("random_actions", |b| {
        b.iter(|| {
            let mut game = decklists::canonical_game().unwrap();
            run_tournament::run_games(
                &mut game,
                10,
                AgentName::PickRandom,
                AgentName::PickRandom,
                RunGames::NoPrint,
            )
            .expect("Error running games");
        })
    });
    group.finish();
}

pub fn minimax_nim(c: &mut Criterion) {
    let mut group = c.benchmark_group("minimax_nim");
    configure(&mut group);
    let state = NimState::new(4);
    let agent = AgentData::omniscient(
        "MINIMAX",
        MinimaxAlgorithm { search_depth: 25 },
        NimWinLossEvaluator {},
    );

    group.bench_function("minimax_nim", |b| {
        b.iter(|| {
            agent.pick_action(agent::deadline(10), &state).expect("Error running agent");
        })
    });
    group.finish();
}

pub fn alpha_beta_nim(c: &mut Criterion) {
    let mut group = c.benchmark_group("alpha_beta_nim");
    configure(&mut group);
    let state = NimState::new(5);
    let agent = AgentData::omniscient(
        "ALPHA_BETA",
        AlphaBetaAlgorithm { search_depth: 25 },
        NimWinLossEvaluator {},
    );

    group.bench_function("alpha_beta_nim", |b| {
        b.iter(|| {
            agent.pick_action(agent::deadline(10), &state).expect("Error running agent");
        })
    });
    group.finish();
}

pub fn uct1_nim(c: &mut Criterion) {
    let mut group = c.benchmark_group("uct1_nim");
    configure(&mut group);
    let state = NimState::new(5);
    let evaluator = RandomPlayoutEvaluator {};
    let player = state.turn;
    let monte_carlo = MonteCarloAlgorithm { child_score_algorithm: Uct1 {} };

    group.bench_function("uct1_nim", |b| {
        b.iter(|| {
            monte_carlo
                .run_search(|i| i == 10_000, &state, &evaluator, player)
                .expect("run_search() Error");
        })
    });
    group.finish();
}

// pub fn uct_search(c: &mut Criterion) {
//     let mut group = c.benchmark_group("uct_search");
//     configure(&mut group);
//     let game = decklists::canonical_game().unwrap();
//     group.bench_function("uct_search", |b| {
//         b.iter(|| monte_carlo::uct_search(&game, Side::Overlord, 1000))
//     });
// }
//
// pub fn alpha_beta_search(c: &mut Criterion) {
//     let mut group = c.benchmark_group("alpha_beta_search");
//     configure(&mut group);
//     let game = decklists::canonical_game().unwrap();
//     group.bench_function("alpha_beta_search", |b| {
//         b.iter(|| alpha_beta::run_search(&game, Side::Overlord, 4).unwrap())
//     });
// }
