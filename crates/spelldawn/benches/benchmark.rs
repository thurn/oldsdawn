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

use ai::agents::{alpha_beta, monte_carlo};
use cards::decklists;
use criterion::{criterion_group, criterion_main, Criterion};
use data::primitives::Side;

criterion_group!(benches, uct_search, alpha_beta_search);
criterion_main!(benches);

pub fn uct_search(c: &mut Criterion) {
    let game = decklists::canonical_game();
    c.bench_function("uct_search", |b| {
        b.iter(|| monte_carlo::uct_search(&game, Side::Overlord, 1000))
    });
}

pub fn alpha_beta_search(c: &mut Criterion) {
    let game = decklists::canonical_game();
    c.bench_function("alpha_beta_search", |b| {
        b.iter(|| alpha_beta::run_search(&game, Side::Overlord, 4).unwrap())
    });
}
