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
use rand::prelude::StdRng;
use rand::SeedableRng;
use rules::{actions, dispatch, mutations};

/// Standard Overlord deck for use in tests
pub static CANONICAL_OVERLORD: Lazy<Deck> = Lazy::new(|| Deck {
    owner_id: PlayerId::new(1),
    identity: CardName::TestOverlordIdentity,
    cards: hashmap! {
        CardName::DungeonAnnex => 3,
        CardName::ActivateReinforcements => 2,
        CardName::ResearchProject => 2,
        CardName::Gemcarver => 2,
        CardName::Coinery => 2,
        CardName::PitTrap => 2,
        CardName::OverwhelmingPower => 2,
        CardName::GatheringDark => 3,
        CardName::ForcedMarch => 2,
        CardName::TimeGolem => 1,
        CardName::TemporalVortex => 2,
        CardName::ShadowLurker => 3,
        CardName::SphinxOfWintersBreath => 2,
        CardName::BridgeTroll => 2,
        CardName::Stormcaller => 2,
        CardName::FireGoblin => 2
    },
});

/// Standard Champion deck for use in tests
pub static CANONICAL_CHAMPION: Lazy<Deck> = Lazy::new(|| Deck {
    owner_id: PlayerId::new(2),
    identity: CardName::TestChampionIdentity,
    cards: hashmap! {
        CardName::Meditation => 2,
        CardName::CoupDeGrace => 3,
        CardName::ChargedStrike => 2,
        CardName::ArcaneRecovery => 3,
        CardName::StealthMission => 2,
        CardName::Preparation => 2,
        CardName::SanctumPassage => 1,
        CardName::Accumulator => 1,
        CardName::MysticPortal => 1,
        CardName::StorageCrystal => 2,
        CardName::MagicalResonator => 2,
        CardName::DarkGrimoire => 1,
        CardName::MaraudersAxe => 2,
        CardName::KeenHalberd => 2,
        CardName::EtherealBlade => 2,
        CardName::BowOfTheAlliance => 2
    },
});

/// Creates a new deterministic game using the canonical decklists, deals
/// opening hands and resolves mulligans.
pub fn canonical_game() -> GameState {
    let mut game = GameState::new(
        GameId::new(0),
        CANONICAL_OVERLORD.clone(),
        CANONICAL_CHAMPION.clone(),
        GameConfiguration { deterministic: true, simulation: true },
    );
    game.rng = Some(StdRng::seed_from_u64(3141592653589793));
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
