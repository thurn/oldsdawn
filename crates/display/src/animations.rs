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
use data::primitives::{CardId, GameObjectId, Side};
use data::special_effects::Projectile;
use data::updates::{GameUpdate, TargetedInteraction};
use data::utils;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{FireProjectileCommand, GameObjectMove, MoveMultipleGameObjectsCommand};

use crate::response_builder::ResponseBuilder;
use crate::{adapters, assets, positions};

pub fn render(
    builder: &mut ResponseBuilder,
    update: &GameUpdate,
    snapshot: &GameState,
) -> Result<()> {
    match update {
        GameUpdate::DrawCards(side, cards) => reveal(builder, *side, cards.iter()),
        GameUpdate::UnveilProject(card_id) => {
            reveal(builder, Side::Champion, vec![*card_id].iter())
        }
        GameUpdate::SummonMinion(card_id) => reveal(builder, Side::Champion, vec![*card_id].iter()),
        GameUpdate::CardsAccessed(_) => {
            // Not sure we need an explicit animation for this?
        }
        GameUpdate::TargetedInteraction(interaction) => {
            targeted_interaction(builder, snapshot, interaction)
        }
    }
    Ok(())
}

fn reveal<'a>(builder: &mut ResponseBuilder, side: Side, cards: impl Iterator<Item = &'a CardId>) {
    if side == builder.user_side {
        builder.push(Command::MoveMultipleGameObjects(MoveMultipleGameObjectsCommand {
            moves: cards
                // Skip animation for cards that are already in a prominent interface position
                .filter(|card_id| !in_display_position(builder, **card_id))
                .enumerate()
                .map(|(i, card_id)| GameObjectMove {
                    id: Some(adapters::game_object_identifier(builder, *card_id)),
                    position: Some(positions::for_sorting_key(
                        i as u32,
                        positions::revealed_cards(),
                    )),
                })
                .collect(),
            disable_animation: !builder.animate,
            delay: Some(adapters::milliseconds(1000)),
        }))
    }
}

fn targeted_interaction(
    builder: &mut ResponseBuilder,
    snapshot: &GameState,
    interaction: &TargetedInteraction,
) {
    let mut projectile = FireProjectileCommand {
        source_id: Some(adapters::game_object_identifier(builder, interaction.source)),
        target_id: Some(adapters::game_object_identifier(builder, interaction.target)),
        projectile: Some(assets::projectile(Projectile::Hovl(3))),
        travel_duration: Some(adapters::milliseconds(300)),
        wait_duration: Some(adapters::milliseconds(300)),
        ..FireProjectileCommand::default()
    };
    apply_projectile(snapshot, &mut projectile, interaction);
    builder.push(Command::FireProjectile(projectile));
}

/// Applies custom projectile effects for a targeted interaction.
fn apply_projectile(
    snapshot: &GameState,
    command: &mut FireProjectileCommand,
    interaction: &TargetedInteraction,
) {
    if let GameObjectId::CardId(card_id) = interaction.source {
        let effects = &rules::card_definition(snapshot, card_id).config.special_effects;
        if let Some(projectile) = effects.projectile {
            command.projectile = Some(assets::projectile(projectile));
        }
        if let Some(additional_hit) = effects.additional_hit {
            command.additional_hit = Some(assets::timed_effect(additional_hit));
            command.additional_hit_delay = Some(adapters::milliseconds(100));
        }
    }
}

fn in_display_position(builder: &ResponseBuilder, card_id: CardId) -> bool {
    utils::is_true(|| {
        Some(matches!(
            builder
                .last_snapshot_positions
                .get(&adapters::card_identifier(card_id))?
                .position
                .as_ref()?,
            Position::Staging(_)
                | Position::Raid(_)
                | Position::Browser(_)
                | Position::Revealed(_)
                | Position::ScoreAnimation(_)
        ))
    })
}
