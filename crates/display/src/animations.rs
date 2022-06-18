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

use anyhow::Result;
use data::game::GameState;
use data::primitives::{CardId, Side};
use data::updates::GameUpdate;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{GameObjectMove, MoveMultipleGameObjectsCommand};

use crate::response_builder::ResponseBuilder;
use crate::{adapters, positions};
use crate::adapters::milliseconds;

pub fn render(builder: &mut ResponseBuilder, update: &GameUpdate, _game: &GameState) -> Result<()> {
    match update {
        GameUpdate::DrawCards(side, cards) => draw_cards(builder, *side, cards),
    }
    Ok(())
}

fn draw_cards(builder: &mut ResponseBuilder, side: Side, cards: &[CardId]) {
    if side == builder.user_side {
        builder.push(Command::MoveMultipleGameObjects(MoveMultipleGameObjectsCommand {
            moves: cards
                .iter()
                .enumerate()
                .map(|(i, card_id)| GameObjectMove {
                    id: Some(adapters::game_object_identifier(*card_id)),
                    position: Some(positions::for_sorting_key(
                        i as u32,
                        positions::revealed_cards(),
                    )),
                })
                .collect(),
            disable_animation: !builder.animate,
            delay: Some(milliseconds(1000))
        }))
    }
}
