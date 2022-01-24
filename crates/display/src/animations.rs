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

//! Functions for turning [GameUpdate]s into animations via a sequence of
//! [GameCommand]s.
//!
//! Animations must be non-essential to the interface state since any changes
//! they make are transient on the client and will be lost if the client
//! reconnects. Non-decorative changes to the client state should be handled by
//! the [full_sync] module.

use data::card_state::CardState;
use data::game::GameState;
use data::primitives::{CardId, RoomId, Side};
use data::special_effects::{
    FantasyEventSounds, FireworksSound, Projectile, SoundEffect, TimedEffect,
};
use data::updates::{GameUpdate, InteractionObjectId, TargetedInteraction};
use protos::spelldawn::game_command::Command;
use protos::spelldawn::game_object_identifier::Id;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::play_effect_position::EffectPosition;
#[allow(unused)] // Used in rustdoc
use protos::spelldawn::{
    game_object_identifier, DelayCommand, DisplayGameMessageCommand, GameCommand, GameMessageType,
    GameObjectIdentifier, MoveGameObjectsCommand, ObjectPosition, ObjectPositionDeck,
    ObjectPositionStaging, PlayerName, RoomVisitType, TimeValue, VisitRoomCommand,
};
use protos::spelldawn::{
    FireProjectileCommand, MusicState, ObjectPositionScoreAnimation, PlayEffectCommand,
    PlayEffectPosition, PlaySoundCommand, SetMusicCommand,
};

use crate::full_sync::CardCreationStrategy;
use crate::response_builder::{CommandPhase, ResponseBuilder};
use crate::{adapters, assets, full_sync};

/// Takes a [GameUpdate] and converts it into an animation, a series of
/// corresponding [GameCommand]s. Commands are appended to the provided
/// `commands` list.
pub fn render(
    commands: &mut ResponseBuilder,
    update: GameUpdate,
    game: &GameState,
    user_side: Side,
) {
    match update {
        GameUpdate::StartTurn(side) => {
            start_turn(commands, side);
        }
        GameUpdate::DrawCard(card_id) => {
            draw_card(commands, game, user_side, card_id);
        }
        GameUpdate::MoveCard(card_id) => {
            move_card(commands, game.card(card_id));
        }
        GameUpdate::RevealCard(card_id) => {
            reveal_card(commands, game, game.card(card_id));
        }
        GameUpdate::InitiateRaid(room_id) => {
            initiate_raid(commands, room_id);
        }
        GameUpdate::TargetedInteraction(interaction) => {
            targeted_interaction(commands, game, user_side, interaction);
        }
        GameUpdate::ChampionScoreCard(card_id, _) => score_champion_card(commands, card_id),
        _ => {}
    }
}

/// Builds a [CardCreationStrategy] for representing the provided `card_id`
/// being drawn.
fn draw_card(commands: &mut ResponseBuilder, game: &GameState, user_side: Side, card_id: CardId) {
    let creation_strategy = if card_id.side == user_side {
        CardCreationStrategy::DrawUserCard
    } else {
        CardCreationStrategy::CreateAtPosition(ObjectPosition {
            sorting_key: u32::MAX,
            position: Some(Position::Deck(ObjectPositionDeck {
                owner: PlayerName::Opponent.into(),
            })),
        })
    };

    commands.push(
        CommandPhase::PreUpdate,
        Command::CreateOrUpdateCard(full_sync::create_or_update_card(
            game,
            game.card(card_id),
            user_side,
            creation_strategy,
        )),
    );

    move_card(commands, game.card(card_id));
}

/// Appends a move card command to move a card to its current location. Skips
/// appending the command if the destination would not be a valid game position,
/// e.g. if it is [CardPosition::DeckUnknown].
fn move_card(commands: &mut ResponseBuilder, card: &CardState) {
    commands.push_optional(
        CommandPhase::Animate,
        full_sync::adapt_position(card, commands.user_side).map(|position| {
            Command::MoveGameObjects(MoveGameObjectsCommand {
                ids: vec![card_id_to_object_id(card.id)],
                position: Some(position),
                disable_animation: false,
            })
        }),
    )
}

/// Commands to reveal the indicated card to all players
fn reveal_card(commands: &mut ResponseBuilder, game: &GameState, card: &CardState) {
    if commands.user_side != card.side
        && game.data.raid.map_or(true, |raid| !card.is_in_room(raid.target))
    {
        // If the hidden card is not part of an active raid, animate it to
        // the staging area on reveal.
        commands.push(
            CommandPhase::Animate,
            Command::MoveGameObjects(MoveGameObjectsCommand {
                ids: vec![card_id_to_object_id(card.id)],
                position: Some(ObjectPosition {
                    sorting_key: 0,
                    position: Some(Position::Staging(ObjectPositionStaging {})),
                }),
                disable_animation: false,
            }),
        );
        commands.push(CommandPhase::Animate, delay(1500));
    }
}

/// Starts the `side` player's turn
fn start_turn(commands: &mut ResponseBuilder, side: Side) {
    commands.push(
        CommandPhase::PostMove,
        Command::DisplayGameMessage(DisplayGameMessageCommand {
            message_type: match side {
                Side::Overlord => GameMessageType::Dusk.into(),
                Side::Champion => GameMessageType::Dawn.into(),
            },
        }),
    )
}

fn initiate_raid(commands: &mut ResponseBuilder, target: RoomId) {
    if commands.user_side == Side::Overlord {
        commands.push(
            CommandPhase::PreUpdate,
            Command::VisitRoom(VisitRoomCommand {
                initiator: adapters::to_player_name(Side::Champion, commands.user_side).into(),
                room_id: adapters::adapt_room_id(target).into(),
                visit_type: RoomVisitType::InitiateRaid.into(),
            }),
        );
        commands.push(CommandPhase::PreUpdate, delay(500));
    }
}

fn targeted_interaction(
    commands: &mut ResponseBuilder,
    game: &GameState,
    user_side: Side,
    interaction: TargetedInteraction,
) {
    let mut projectile = FireProjectileCommand {
        source_id: Some(adapt_interaction_id(interaction.source, user_side)),
        target_id: Some(adapt_interaction_id(interaction.target, user_side)),
        projectile: Some(assets::projectile(Projectile::Hovl(3))),
        travel_duration: Some(duration_ms(300)),
        wait_duration: Some(duration_ms(300)),
        ..FireProjectileCommand::default()
    };
    apply_projectile(game, &mut projectile, interaction);
    commands.push(CommandPhase::PreUpdate, Command::FireProjectile(projectile));
}

/// Applies custom projectile effects for a targeted interaction.
fn apply_projectile(
    game: &GameState,
    command: &mut FireProjectileCommand,
    interaction: TargetedInteraction,
) {
    if let (InteractionObjectId::CardId(card_id), _) = (interaction.source, interaction.target) {
        let effects = &rules::card_definition(game, card_id).config.special_effects;
        if let Some(projectile) = effects.projectile {
            command.projectile = Some(assets::projectile(projectile));
        }
        if let Some(additional_hit) = effects.additional_hit {
            command.additional_hit = Some(assets::timed_effect(additional_hit));
            command.additional_hit_delay = Some(duration_ms(100));
        }
    }
}

fn score_champion_card(commands: &mut ResponseBuilder, card_id: CardId) {
    let object_id = card_id_to_object_id(card_id);
    commands.push(CommandPhase::PreUpdate, set_music(MusicState::Silent));
    commands.push(
        CommandPhase::PreUpdate,
        play_sound(SoundEffect::FantasyEvents(FantasyEventSounds::Positive1)),
    );
    commands.push(
        CommandPhase::PreUpdate,
        move_to(
            object_id,
            ObjectPosition {
                sorting_key: 0,
                position: Some(Position::ScoreAnimation(ObjectPositionScoreAnimation {})),
            },
        ),
    );
    commands.push(
        CommandPhase::PreUpdate,
        play_effect(
            TimedEffect::HovlMagicHit(4),
            object_id,
            PlayEffectOptions {
                duration: Some(duration_ms(700)),
                sound: Some(SoundEffect::Fireworks(FireworksSound::RocketExplodeLarge)),
                ..PlayEffectOptions::default()
            },
        ),
    );
    commands.push(
        CommandPhase::PreUpdate,
        play_effect(
            TimedEffect::HovlMagicHit(4),
            object_id,
            PlayEffectOptions {
                duration: Some(duration_ms(300)),
                sound: Some(SoundEffect::Fireworks(FireworksSound::RocketExplode)),
                ..PlayEffectOptions::default()
            },
        ),
    );
}

/// Constructs a delay command
fn delay(milliseconds: u32) -> Command {
    Command::Delay(DelayCommand { duration: Some(TimeValue { milliseconds }) })
}

fn set_music(music_state: MusicState) -> Command {
    Command::SetMusic(SetMusicCommand { music_state: music_state.into() })
}

fn play_sound(sound: SoundEffect) -> Command {
    Command::PlaySound(PlaySoundCommand { sound: Some(assets::sound_effect(sound)) })
}

fn move_to(object: GameObjectIdentifier, position: ObjectPosition) -> Command {
    Command::MoveGameObjects(MoveGameObjectsCommand {
        ids: vec![object],
        position: Some(position),
        disable_animation: false,
    })
}

#[derive(Debug, Default)]
struct PlayEffectOptions {
    pub duration: Option<TimeValue>,
    pub sound: Option<SoundEffect>,
    pub scale: Option<f32>,
}

fn play_effect(
    effect: TimedEffect,
    object: GameObjectIdentifier,
    options: PlayEffectOptions,
) -> Command {
    Command::PlayEffect(PlayEffectCommand {
        effect: Some(assets::timed_effect(effect)),
        position: Some(PlayEffectPosition {
            effect_position: Some(EffectPosition::GameObject(object)),
        }),
        scale: options.scale,
        duration: Some(options.duration.unwrap_or_else(|| duration_ms(300))),
        sound: options.sound.map(assets::sound_effect),
    })
}

/// Constructs a [TimeValue].
fn duration_ms(milliseconds: u32) -> TimeValue {
    TimeValue { milliseconds }
}

/// Converts a [CardId] into a client [GameObjectIdentifier]
fn card_id_to_object_id(id: CardId) -> GameObjectIdentifier {
    GameObjectIdentifier { id: Some(Id::CardId(adapters::adapt_card_id(id))) }
}

fn adapt_interaction_id(id: InteractionObjectId, user_side: Side) -> GameObjectIdentifier {
    GameObjectIdentifier {
        id: Some(match id {
            InteractionObjectId::CardId(id) => Id::CardId(adapters::adapt_card_id(id)),
            InteractionObjectId::Identity(side) => {
                Id::Identity(adapters::to_player_name(side, user_side).into())
            }
            InteractionObjectId::Deck(side) => {
                Id::Deck(adapters::to_player_name(side, user_side).into())
            }
            InteractionObjectId::DiscardPile(side) => {
                Id::DiscardPile(adapters::to_player_name(side, user_side).into())
            }
        }),
    }
}
