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

use data::card_name::CardName;
use data::deck::Deck;
use data::game::{GameConfiguration, GameState, MulliganDecision};
use data::game_actions::{PromptAction, UserAction};
use data::primitives::{GameId, PlayerId, Side};
use maplit::hashmap;
use once_cell::sync::Lazy;
use rules::{actions, dispatch, mutations};

/// Standard Overlord deck for use in tests
pub static CANONICAL_OVERLORD: Lazy<Deck> = Lazy::new(|| Deck {
    owner_id: PlayerId::new(1),
    identity: CardName::TestOverlordIdentity,
    cards: hashmap! {
        CardName::DungeonAnnex => 15,
        CardName::IceDragon => 15,
        CardName::GoldMine => 15
    },
});

/// Standard Champion deck for use in tests
pub static CANONICAL_CHAMPION: Lazy<Deck> = Lazy::new(|| Deck {
    owner_id: PlayerId::new(2),
    identity: CardName::TestChampionIdentity,
    cards: hashmap! {
        CardName::Lodestone => 15,
        CardName::Greataxe => 15,
        CardName::ArcaneRecovery => 15,
    },
});

/// Creates a new game using the canonical decklists, deals opening hands and
/// resolves mulligans.
pub fn canonical_game() -> GameState {
    crate::initialize();
    let mut game = GameState::new(
        GameId::new(0),
        CANONICAL_OVERLORD.clone(),
        CANONICAL_CHAMPION.clone(),
        GameConfiguration { deterministic: false, simulation: true },
    );
    dispatch::populate_delegate_cache(&mut game);
    mutations::deal_opening_hands(&mut game);
    actions::handle_user_action(
        &mut game,
        Side::Overlord,
        UserAction::GamePromptResponse(PromptAction::MulliganDecision(MulliganDecision::Keep)),
    )
    .unwrap();
    actions::handle_user_action(
        &mut game,
        Side::Champion,
        UserAction::GamePromptResponse(PromptAction::MulliganDecision(MulliganDecision::Keep)),
    )
    .unwrap();

    game
}
