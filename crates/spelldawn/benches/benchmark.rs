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

use ai::agents::monte_carlo;
use criterion::{criterion_group, criterion_main, Criterion};
use data::card_name::CardName;
use data::deck::Deck;
use data::game::{GameConfiguration, GameState, MulliganDecision};
use data::game_actions::{PromptAction, UserAction};
use data::primitives::{GameId, PlayerId, Side};
use maplit::hashmap;
use rules::{actions, mutations, DEFINITIONS};

fn new_game() -> GameState {
    let count = cards::initialize();
    println!("Found {:?} cards for {:?}", count, DEFINITIONS.len());

    let mut game = GameState::new(
        GameId::new(0),
        Deck {
            owner_id: PlayerId::new(1),
            identity: CardName::TestOverlordIdentity,
            cards: hashmap! {
                CardName::DungeonAnnex => 15,
                CardName::IceDragon => 15,
                CardName::GoldMine => 15
            },
        },
        Deck {
            owner_id: PlayerId::new(2),
            identity: CardName::TestChampionIdentity,
            cards: hashmap! {
                CardName::Lodestone => 15,
                CardName::Greataxe => 15,
                CardName::ArcaneRecovery => 15,
            },
        },
        GameConfiguration { deterministic: false, simulation: true },
    );
    mutations::deal_opening_hands(&mut game);
    actions::handle_user_action(
        &mut game,
        Side::Overlord,
        UserAction::PromptResponse(PromptAction::MulliganDecision(MulliganDecision::Keep)),
    )
    .unwrap();
    actions::handle_user_action(
        &mut game,
        Side::Champion,
        UserAction::PromptResponse(PromptAction::MulliganDecision(MulliganDecision::Keep)),
    )
    .unwrap();
    actions::handle_user_action(&mut game, Side::Overlord, UserAction::DrawCard).unwrap();
    game
}

pub fn uct_search(c: &mut Criterion) {
    let game = new_game();
    c.bench_function("uct search", |b| {
        b.iter(|| monte_carlo::uct_search(&game, Side::Overlord, 100))
    });
}

criterion_group!(benches, uct_search);
criterion_main!(benches);
