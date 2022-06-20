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
use data::primitives::{AbilityId, CardId, GameObjectId, RoomId, Side};
use data::special_effects::{
    FantasyEventSounds, FireworksSound, Projectile, SoundEffect, TimedEffect,
};
use data::updates::{GameUpdate, TargetedInteraction};
use data::utils;
use fallible_iterator::FallibleIterator;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::play_effect_position::EffectPosition;
use protos::spelldawn::{
    CreateTokenCardCommand, DelayCommand, DisplayGameMessageCommand, DisplayRewardsCommand,
    FireProjectileCommand, GameMessageType, GameObjectMove, MoveMultipleGameObjectsCommand,
    MusicState, PlayEffectCommand, PlayEffectPosition, PlaySoundCommand, RoomVisitType,
    SetGameObjectsEnabledCommand, SetMusicCommand, TimeValue, VisitRoomCommand,
};

use crate::response_builder::ResponseBuilder;
use crate::{adapters, assets, card_sync, positions};

pub fn render(
    builder: &mut ResponseBuilder,
    update: &GameUpdate,
    snapshot: &GameState,
) -> Result<()> {
    match update {
        GameUpdate::StartTurn(side) => start_turn(builder, *side),
        GameUpdate::PlayCardFaceUp(side, card_id) => {
            reveal(builder, side.opponent(), &vec![*card_id])
        }
        GameUpdate::AbilityActivated(side, ability_id) => {
            if *side != builder.user_side {
                show_ability(builder, snapshot, *ability_id);
            }
        }
        GameUpdate::AbilityTriggered(ability_id) => show_ability(builder, snapshot, *ability_id),
        GameUpdate::DrawCards(side, cards) => reveal(builder, *side, cards),
        GameUpdate::ShuffleIntoDeck => {
            // No animation, just acts as a snapshot point.
        }
        GameUpdate::UnveilProject(card_id) => reveal(builder, Side::Champion, &vec![*card_id]),
        GameUpdate::SummonMinion(card_id) => reveal(builder, Side::Champion, &vec![*card_id]),
        GameUpdate::LevelUpRoom(room_id) => level_up_room(builder, *room_id),
        GameUpdate::InitiateRaid(room_id) => initiate_raid(builder, *room_id),
        GameUpdate::TargetedInteraction(interaction) => {
            targeted_interaction(builder, snapshot, interaction)
        }
        GameUpdate::ScoreCard(_, card_id) => score_card(builder, *card_id),
        GameUpdate::GameOver(side) => game_over(builder, snapshot, *side)?,
    }
    Ok(())
}

fn start_turn(builder: &mut ResponseBuilder, side: Side) {
    builder.push(Command::DisplayGameMessage(DisplayGameMessageCommand {
        message_type: match side {
            Side::Overlord => GameMessageType::Dusk.into(),
            Side::Champion => GameMessageType::Dawn.into(),
        },
    }))
}

fn reveal(builder: &mut ResponseBuilder, revealed_to: Side, cards: &Vec<CardId>) {
    let is_large_draw = cards.len() >= 4;
    if revealed_to == builder.user_side {
        builder.push(Command::MoveMultipleGameObjects(MoveMultipleGameObjectsCommand {
            moves: cards
                .iter()
                // Skip animation for cards that are already in a prominent interface position
                .filter(|card_id| !in_display_position(builder, **card_id))
                .enumerate()
                .map(|(i, card_id)| GameObjectMove {
                    id: Some(adapters::game_object_identifier(builder, *card_id)),
                    position: Some(positions::for_sorting_key(
                        i as u32,
                        if is_large_draw {
                            positions::browser()
                        } else {
                            positions::revealed_cards()
                        },
                    )),
                })
                .collect(),
            disable_animation: !builder.animate,
            delay: Some(adapters::milliseconds(if is_large_draw { 2000 } else { 1000 })),
        }))
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

fn show_ability(builder: &mut ResponseBuilder, snapshot: &GameState, ability_id: AbilityId) {
    let mut card = card_sync::ability_card_view(builder, snapshot, ability_id, None);
    card.card_position = Some(positions::for_ability(snapshot, ability_id, positions::staging()));

    builder.push(Command::CreateTokenCard(CreateTokenCardCommand {
        card: Some(card),
        animate: builder.animate,
    }));

    builder.push(delay(1500));
}

fn level_up_room(commands: &mut ResponseBuilder, target: RoomId) {
    commands.push(Command::VisitRoom(VisitRoomCommand {
        initiator: commands.to_player_name(Side::Overlord),
        room_id: adapters::room_identifier(target),
        visit_type: RoomVisitType::LevelUpRoom.into(),
    }));
}

fn initiate_raid(commands: &mut ResponseBuilder, target: RoomId) {
    commands.push(Command::VisitRoom(VisitRoomCommand {
        initiator: commands.to_player_name(Side::Champion),
        room_id: adapters::room_identifier(target),
        visit_type: RoomVisitType::InitiateRaid.into(),
    }));
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

fn score_card(builder: &mut ResponseBuilder, card_id: CardId) {
    builder.push(set_music(MusicState::Silent));
    builder.push(play_sound(SoundEffect::FantasyEvents(FantasyEventSounds::Positive1)));
    builder.push(play_effect(
        builder,
        TimedEffect::HovlMagicHit(4),
        card_id,
        PlayEffectOptions {
            duration: Some(adapters::milliseconds(700)),
            sound: Some(SoundEffect::Fireworks(FireworksSound::RocketExplodeLarge)),
            ..PlayEffectOptions::default()
        },
    ));
    builder.push(play_effect(
        builder,
        TimedEffect::HovlMagicHit(4),
        card_id,
        PlayEffectOptions {
            duration: Some(adapters::milliseconds(300)),
            sound: Some(SoundEffect::Fireworks(FireworksSound::RocketExplode)),
            ..PlayEffectOptions::default()
        },
    ));
    builder.push(delay(1000));
}

fn game_over(builder: &mut ResponseBuilder, snapshot: &GameState, winner: Side) -> Result<()> {
    builder.push(delay(1000));

    builder.push(Command::SetGameObjectsEnabled(SetGameObjectsEnabledCommand {
        game_objects_enabled: false,
    }));

    builder.push(Command::DisplayGameMessage(DisplayGameMessageCommand {
        message_type: if winner == builder.user_side {
            GameMessageType::Victory
        } else {
            GameMessageType::Defeat
        }
        .into(),
    }));

    if winner == builder.user_side {
        // TODO: Show real rewards instead of placeholder values
        builder.push(Command::DisplayRewards(DisplayRewardsCommand {
            rewards: utils::fallible(
                snapshot
                    .cards(winner)
                    .iter()
                    .filter(|card| card.is_revealed_to(winner) && !card.position().is_identity())
                    .take(5),
            )
            .map(|card| card_sync::card_view(builder, snapshot, card))
            .collect()?,
        }));
    }

    Ok(())
}

#[derive(Debug, Default)]
struct PlayEffectOptions {
    pub duration: Option<TimeValue>,
    pub sound: Option<SoundEffect>,
    pub scale: Option<f32>,
}

fn play_effect(
    builder: &ResponseBuilder,
    effect: TimedEffect,
    id: impl Into<GameObjectId>,
    options: PlayEffectOptions,
) -> Command {
    Command::PlayEffect(PlayEffectCommand {
        effect: Some(assets::timed_effect(effect)),
        position: Some(PlayEffectPosition {
            effect_position: Some(EffectPosition::GameObject(adapters::game_object_identifier(
                builder,
                id.into(),
            ))),
        }),
        scale: options.scale,
        duration: Some(options.duration.unwrap_or_else(|| adapters::milliseconds(300))),
        sound: options.sound.map(assets::sound_effect),
    })
}

fn delay(milliseconds: u32) -> Command {
    Command::Delay(DelayCommand { duration: Some(TimeValue { milliseconds }) })
}

fn set_music(music_state: MusicState) -> Command {
    Command::SetMusic(SetMusicCommand { music_state: music_state.into() })
}

fn play_sound(sound: SoundEffect) -> Command {
    Command::PlaySound(PlaySoundCommand { sound: Some(assets::sound_effect(sound)) })
}
