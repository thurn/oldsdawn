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
#![deny(clippy::unwrap_in_result)]
#![deny(clippy::unwrap_used)]
#![deny(clippy::use_self)]
#![deny(clippy::used_underscore_binding)]
#![deny(clippy::useless_let_if_seq)]
#![deny(clippy::wildcard_imports)]
#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

pub mod client;

use data::card_name::CardName;
use data::game::GameState;
use data::primitives::{ActionCount, ManaValue, PointsValue, RoomId, Side};

/// Creates a [new_game], setting the turn to [Side::Champion] and the available mana to `mana`.
pub fn new_champion_game_with_mana(mana: ManaValue) -> GameState {
    new_game(Side::Champion, NewGameConfig { mana, ..NewGameConfig::default() })
}

/// Creates a [new_game], setting the turn to [Side::Overlord] and the available mana to `mana`.
pub fn new_overlord_game_with_mana(mana: ManaValue) -> GameState {
    new_game(Side::Overlord, NewGameConfig { mana, ..NewGameConfig::default() })
}

/// Creates a new game on the `turn` player's first turn. By default this is very similar to the
/// state of a normal new game, see [NewGameConfig] for information about the default configuration
/// options and how to modify them.
pub fn new_game(turn: Side, config: NewGameConfig) -> GameState {
    todo!()
}

#[derive(Clone, Debug)]
pub struct NewGameConfig {
    /// Mana available for the `turn` player. Defaults to 5.
    pub mana: ManaValue,
    /// Actions available for the `turn` player. Defaults to 3.
    pub actions: ActionCount,
    /// Score for the `turn` player. Defaults to 0.
    pub points: PointsValue,
}

impl Default for NewGameConfig {
    fn default() -> Self {
        Self { mana: 5, actions: 3, points: 0 }
    }
}

/// Draws and plays a card from hand by name. This is similar to the normal 'play card' action
/// during a game except that it replaces a test card in the player's deck with the indicated card
/// name and then draws that card before playing it.
///
/// All normal restrictions on card playing apply, e.g. it must currently be the card owner's main
/// phase and they must have sufficient mana & action points available to play it. The card will be
/// played into [RoomId::RoomA] if a room target is required for this card type.
pub fn play_from_hand(game: &mut GameState, card: CardName) {
    todo!()
}
