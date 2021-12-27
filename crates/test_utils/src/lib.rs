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

#![deny(warnings)]
#![deny(clippy::all)]
#![deny(clippy::cast_lossless)]
#![deny(clippy::cloned_instead_of_copied)]
#![deny(clippy::copy_iterator)]
#![deny(clippy::default_trait_access)]
#![deny(clippy::if_then_some_else_none)]
#![deny(clippy::inconsistent_struct_constructor)]
#![deny(clippy::inefficient_to_string)]
#![deny(clippy::integer_division)]
#![deny(clippy::let_underscore_drop)]
#![deny(clippy::let_underscore_must_use)]
#![deny(clippy::manual_ok_or)]
#![deny(clippy::map_flatten)]
#![deny(clippy::map_unwrap_or)]
#![deny(clippy::match_same_arms)]
#![deny(clippy::multiple_inherent_impl)]
#![deny(clippy::needless_continue)]
#![deny(clippy::needless_for_each)]
#![deny(clippy::option_if_let_else)]
#![deny(clippy::redundant_closure_for_method_calls)]
#![deny(clippy::ref_option_ref)]
#![deny(clippy::string_to_string)]
#![deny(clippy::trait_duplication_in_bounds)]
#![deny(clippy::unnecessary_self_imports)]
#![deny(clippy::unnested_or_patterns)]
#![deny(clippy::unused_self)]
#![deny(clippy::use_self)]
#![deny(clippy::used_underscore_binding)]
#![deny(clippy::useless_let_if_seq)]
#![deny(clippy::wildcard_imports)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod client;

use data::card_name::CardName;
use data::deck::Deck;
use data::game::{GameState, NewGameOptions};
use data::primitives::{ActionCount, CardType, ManaValue, PointsValue, RoomId, Side};
use display::rendering;
use maplit::hashmap;
use protos::spelldawn::game_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{card_target, CardTarget, PlayCardAction};

use crate::client::TestGame;

/// Creates a new game with the user playing as the `user_side` player. By default, this creates a
/// normal new game with both player's decks populated with blank test cards. The game is advanced
/// to the user's first turn. See [Args] for information about the default configuration options
/// and how to modify them.
pub fn new_game(user_side: Side, args: Args) -> TestGame {
    let (overlord_user, champion_user) = match user_side {
        Side::Overlord => (TestGame::USER_ID, TestGame::OPPONENT_ID),
        Side::Champion => (TestGame::OPPONENT_ID, TestGame::USER_ID),
    };

    let mut state = GameState::new_game(
        TestGame::GAME_ID,
        Deck {
            owner_id: overlord_user,
            identity: CardName::TestOverlordIdentity,
            cards: hashmap! {CardName::TestOverlordSpell => 45},
        },
        Deck {
            owner_id: champion_user,
            identity: CardName::TestChampionIdentity,
            cards: hashmap! {CardName::TestChampionSpell => 45},
        },
        NewGameOptions::default(),
    );

    state.data.turn = user_side;
    state.player_mut(user_side).mana = args.mana;
    state.player_mut(user_side).actions = args.actions;
    state.player_mut(user_side).score = args.score;

    TestGame::new(state, user_side)
}

#[derive(Clone, Debug)]
pub struct Args {
    /// Mana available for the `user_side` player. Defaults to 5.
    pub mana: ManaValue,
    /// Actions available for the `user_side` player. Defaults to 3.
    pub actions: ActionCount,
    /// Score for the `user_side` player. Defaults to 0.
    pub score: PointsValue,
}

impl Default for Args {
    fn default() -> Self {
        Self { mana: 5, actions: 3, score: 0 }
    }
}

/// Draws and then plays a named card.
///
/// This function first draws a copy of the requested card from the user's deck via
/// [TestGame::draw_named_card]. The card is then played via the standard [PlayCardAction]. If the
/// card is a minion, project, scheme, or upgrade card, it is played into the [TestGame::ROOM_ID]
/// room. A list of the [Command]s produced by playing the card is returned.
pub fn play_from_hand(game: &mut TestGame, card_name: CardName) -> Vec<Command> {
    let card_id = game.draw_named_card(card_name);

    let target = match rules::get(card_name).card_type {
        CardType::Minion | CardType::Project | CardType::Scheme | CardType::Upgrade => {
            Some(CardTarget {
                card_target: Some(card_target::CardTarget::RoomId(
                    rendering::adapt_room_id(TestGame::ROOM_ID).into(),
                )),
            })
        }
        _ => None,
    };

    game.perform_action(Action::PlayCard(PlayCardAction { card_id: Some(card_id), target }))
}
