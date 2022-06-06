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

use ai::agents::{alpha_beta, monte_carlo};
use ai::tournament::run_tournament;
use ai::tournament::run_tournament::RunGames;
use cards::{decklists, initialize};
use criterion::measurement::WallTime;
use criterion::{criterion_group, criterion_main, BenchmarkGroup, Criterion};
use data::agent_definition::AgentName;
use data::primitives::Side;

criterion_group!(benches, random_actions, uct_search, alpha_beta_search);
criterion_main!(benches);

fn configure(group: &mut BenchmarkGroup<WallTime>) {
    group.confidence_level(0.99).noise_threshold(0.025).measurement_time(Duration::from_secs(60));
}

pub fn random_actions(c: &mut Criterion) {
    initialize::run();
    let mut group = c.benchmark_group("random_actions");
    configure(&mut group);
    group.bench_function("random_actions", |b| {
        b.iter(|| {
            let mut game = decklists::canonical_game();
            run_tournament::run_games(
                &mut game,
                10,
                AgentName::PickRandom,
                AgentName::PickRandom,
                RunGames::NoPrint,
            );
        })
    });
    group.finish();
}

pub fn uct_search(c: &mut Criterion) {
    initialize::run();
    let mut group = c.benchmark_group("uct_search");
    configure(&mut group);
    let game = decklists::canonical_game();
    group.bench_function("uct_search", |b| {
        b.iter(|| monte_carlo::uct_search(&game, Side::Overlord, 1000))
    });
}

pub fn alpha_beta_search(c: &mut Criterion) {
    initialize::run();
    let mut group = c.benchmark_group("alpha_beta_search");
    configure(&mut group);
    let game = decklists::canonical_game();
    group.bench_function("alpha_beta_search", |b| {
        b.iter(|| alpha_beta::run_search(&game, Side::Overlord, 4).unwrap())
    });
}
