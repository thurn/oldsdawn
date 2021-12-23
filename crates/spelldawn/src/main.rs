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

pub mod database;
pub mod server;

use data::{game::{GameState, NewGameOptions, RaidState}, card_state::CardPositionKind};
use data::primitives::{
    AbilityId, AbilityIndex, BoostData, CardId, GameId, RaidId, RoomId, RoomLocation, Side, UserId,
};
use data::{
    card_definition::{Ability, CardDefinition},
};
use tonic::{transport::Server, Request, Response, Status};

use protos::spelldawn::game_command::Command;
use protos::spelldawn::spelldawn_server::{Spelldawn, SpelldawnServer};
use protos::spelldawn::{CommandList, GameCommand, GameRequest, GameView, UpdateGameViewCommand};

use data::card_name::CardName;
use data::card_state::{CardPosition, CardState};
use data::delegates;
use data::delegates::{Delegate, Scope};
use rules::{dispatch, helpers, queries, CARDS};

use crate::server::GameService;
use data::card_name::CardName::TestOverlordIdentity;
use data::deck::Deck;
use maplit::hashmap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Num CARDS {:?}", CARDS.len());

    let mut game = GameState::new_game(
        GameId::new(1),
        Deck {
            owner_id: UserId::new(2),
            identity: CardName::TestOverlordIdentity,
            cards: hashmap! {
                CardName::DungeonAnnex => 1,
                CardName::GoldMine => 1,
                CardName::IceDragon => 1
            },
        },
        Deck {
            owner_id: UserId::new(1),
            identity: CardName::TestChampionIdentity,
            cards: hashmap! {
                CardName::ArcaneRecovery => 1,
                CardName::Greataxe => 1
            },
        },
        NewGameOptions::default(),
    );

    let arcane_recovery = CardId::new(Side::Champion, 1);
    println!("Arcane Recovery. Starting Mana: {:?}", game.player(Side::Champion).mana);
    dispatch::invoke_event(&mut game, delegates::on_play_card, arcane_recovery);
    println!("Updated Mana: {:?}", game.player(Side::Champion).mana);

    let greataxe = CardId::new(Side::Champion, 2);
    println!("Greataxe. Starting Attack: {:?}", queries::attack(&game, greataxe));
    dispatch::invoke_event(
        &mut game,
        delegates::on_activate_boost,
        BoostData { card_id: greataxe, count: 2 },
    );
    println!("Greataxe. Updated Attack: {:?}", queries::attack(&game, greataxe));

    let gold_mine = CardId::new(Side::Overlord, 2);
    game.move_card(gold_mine, CardPosition::Room(RoomId::RoomA, RoomLocation::InRoom));
    dispatch::invoke_event(&mut game, delegates::on_play_card, gold_mine);
    println!("Gold Mine. Starting Stored Mana: {:?}", game.card(gold_mine).data.stored_mana);
    dispatch::invoke_event(&mut game, delegates::on_dusk, 1);
    println!(
        "Gold Mine. Stored Mana: {:?} Overlord Mana: {:?}",
        game.card(gold_mine).data.stored_mana,
        game.player(Side::Overlord).mana
    );
    dispatch::invoke_event(&mut game, delegates::on_dusk, 1);
    dispatch::invoke_event(&mut game, delegates::on_dusk, 1);
    dispatch::invoke_event(&mut game, delegates::on_dusk, 1);
    dispatch::invoke_event(&mut game, delegates::on_dusk, 1);
    println!(
        "Gold Mine. Stored Mana: {:?} Overlord Mana: {:?} Card Position: {:?}",
        game.card(gold_mine).data.stored_mana,
        game.player(Side::Overlord).mana,
        game.card(gold_mine).position
    );

    let ice_dragon = CardId::new(Side::Overlord, 3);
    game.move_card(arcane_recovery, CardPosition::Hand(Side::Champion));
    game.move_card(ice_dragon, CardPosition::Room(RoomId::RoomB, RoomLocation::Defender));
    game.data.raid =
        Some(RaidState { raid_id: RaidId(0), encounter_number: 0, priority: Side::Overlord });
    println!(
        "Ice Dragon. Starting Hand Size: {:?}. Raid: {:?}",
        game.hand(Side::Champion).count(),
        game.data.raid
    );
    dispatch::invoke_event(&mut game, delegates::on_minion_combat_ability, ice_dragon);
    println!(
        "Ice Dragon. Hand Size: {:?}. Raid: {:?}.",
        game.hand(Side::Champion).count(),
        game.data.raid
    );

    println!("Dungeon Annex. Starting Mana: {:?}", game.player(Side::Overlord).mana);
    let dungeon_annex = CardId::new(Side::Overlord, 1);
    dispatch::invoke_event(&mut game, delegates::on_score_scheme, dungeon_annex);
    println!("Dungeon Annex. Resulting Mana: {:?}", game.player(Side::Overlord).mana);

    let address = "127.0.0.1:50052".parse().expect("valid address");
    let service = tonic_web::config()
        .allow_origins(vec!["127.0.0.1"])
        .enable(SpelldawnServer::new(GameService::default()));
    println!("Server listening on {}", address);
    Server::builder().accept_http1(true).add_service(service).serve(address).await?;

    Ok(())
}
