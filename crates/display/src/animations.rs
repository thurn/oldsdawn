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

use data::card_state::{CardPosition, CardState};
use data::game::GameState;
use data::primitives::{AbilityId, CardId, RoomId, Side};
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
    CardCreationAnimation, CreateOrUpdateCardCommand, DestroyCardCommand, DisplayRewardsCommand,
    FireProjectileCommand, MusicState, ObjectPositionBrowser, ObjectPositionHand,
    ObjectPositionIdentity, ObjectPositionIdentityContainer, ObjectPositionIntoCard,
    ObjectPositionScoreAnimation, PlayEffectCommand, PlayEffectPosition, PlaySoundCommand,
    SetGameObjectsEnabledCommand, SetMusicCommand,
};
use rules::flags;

use crate::full_sync::CardCreationStrategy;
use crate::response_builder::{CardUpdateTypes, ResponseBuilder, UpdateType};
use crate::{adapters, assets, full_sync};

pub fn populate_card_update_types(
    game: &GameState,
    user_side: Side,
    update: &GameUpdate,
    types: &mut CardUpdateTypes,
) {
    match update {
        GameUpdate::DrawHand(side) => {
            for card in game.hand(*side) {
                types.insert(card.id, UpdateType::Animation);
            }
        }
        GameUpdate::KeepHand(_, cards) => {
            for card_id in cards {
                types.insert(*card_id, UpdateType::Animation);
            }
        }
        GameUpdate::DrawCard(card_id) => {
            types.insert(*card_id, UpdateType::Utility);
        }
        GameUpdate::MulliganHand(_, old_cards, new_cards) => {
            for card_id in old_cards {
                types.insert(*card_id, UpdateType::Animation);
            }
            for card_id in new_cards {
                types.insert(*card_id, UpdateType::Animation);
            }
        }
        GameUpdate::ShuffleIntoDeck(card_id) => {
            types.insert(*card_id, UpdateType::Utility);
        }
        GameUpdate::DestroyCard(card_id) => {
            types.insert(*card_id, UpdateType::Utility);
        }
        GameUpdate::OverlordScoreCard(card_id, _) => {
            types.insert(*card_id, UpdateType::Animation);
        }
        GameUpdate::ChampionScoreCard(card_id, _) => {
            types.insert(*card_id, UpdateType::Animation);
        }
        GameUpdate::RevealToOpponent(card_id) => {
            if game.data.raid.is_none() && user_side != game.card(*card_id).side() {
                // Kind of a hack: reveal_card only adds animations for the opponent when no
                // raid is active.
                // TODO: Update this logic to be less magical
                types.insert(*card_id, UpdateType::Reveal);
            }
        }
        _ => {}
    }
}

/// Takes a [GameUpdate] and converts it into an animation, a series of
/// corresponding [GameCommand]s. Commands are appended to the provided
/// `commands` list.
pub fn render(
    commands: &mut ResponseBuilder,
    update: &GameUpdate,
    game: &GameState,
    user_side: Side,
) {
    match update {
        GameUpdate::LevelUpRoom(room_id) => level_up_room(commands, *room_id),
        GameUpdate::InitiateRaid(room_id) => initiate_raid(commands, *room_id),
        GameUpdate::StartTurn(side) => start_turn(commands, *side),
        GameUpdate::DrawHand(side) => draw_hand(commands, game, *side),
        GameUpdate::KeepHand(side, cards) => keep_hand(commands, game, *side, cards),
        GameUpdate::MulliganHand(side, old_cards, new_cards) => {
            mulligan_hand(commands, game, *side, old_cards, new_cards)
        }
        GameUpdate::DrawCard(card_id) => draw_card(commands, game, user_side, *card_id),
        GameUpdate::RevealToOpponent(card_id) => reveal_card(commands, game, game.card(*card_id)),
        GameUpdate::TargetedInteraction(interaction) => {
            targeted_interaction(commands, game, user_side, *interaction)
        }
        GameUpdate::OverlordScoreCard(card_id, _) => {
            score_card(commands, game, game.card(*card_id), Side::Overlord)
        }
        GameUpdate::ChampionScoreCard(card_id, _) => {
            score_card(commands, game, game.card(*card_id), Side::Champion)
        }
        GameUpdate::DestroyCard(card_id) => destroy_card(commands, UpdateType::Utility, *card_id),
        GameUpdate::MoveToZone(card_id) => {
            move_card(commands, UpdateType::Utility, game.card(*card_id))
        }
        GameUpdate::ShuffleIntoDeck(card_id) => {
            shuffle_into_deck(commands, game, user_side, *card_id, UpdateType::Utility)
        }
        GameUpdate::AbilityActivated(ability_id) => {
            if ability_id.card_id.side == commands.user_side {
                if flags::can_take_activate_ability_action(game, commands.user_side, *ability_id) {
                    // Ability cards are moved into their owning card when played, but the diff
                    // algorithm doesn't know about this happening. So we need
                    // to return them to hand if they are still playable.
                    move_ability_to_hand(commands, game, *ability_id);
                }
            } else {
                show_ability_fired(commands, game, *ability_id)
            }
        }
        GameUpdate::AbilityTriggered(ability_id) => show_ability_fired(commands, game, *ability_id),
        GameUpdate::GameOver(side) => game_over(commands, game, *side),
        _ => todo!("Implement {:?}", update),
    }
}

fn draw_hand(commands: &mut ResponseBuilder, game: &GameState, side: Side) {
    let hand = game.card_list_for_position(side, CardPosition::Hand(side));
    for card in hand {
        commands.push(
            UpdateType::Animation,
            Command::CreateOrUpdateCard(full_sync::create_or_update_card(
                game,
                game.card(card.id),
                commands.user_side,
                CardCreationStrategy::CreateAtPosition(ObjectPosition {
                    sorting_key: card.sorting_key,
                    position: Some(Position::Deck(ObjectPositionDeck {
                        owner: adapters::to_player_name(side, commands.user_side).into(),
                    })),
                }),
            )),
        );

        if commands.user_side == side {
            commands.push(
                UpdateType::Animation,
                Command::MoveGameObjects(MoveGameObjectsCommand {
                    ids: vec![adapters::card_id_to_object_id(card.id)],
                    position: Some(ObjectPosition {
                        sorting_key: card.sorting_key,
                        position: Some(Position::Browser(ObjectPositionBrowser {})),
                    }),
                    disable_animation: false,
                }),
            );
        }

        commands.move_card(
            UpdateType::Animation,
            card,
            Position::Hand(ObjectPositionHand {
                owner: adapters::to_player_name(side, commands.user_side).into(),
            }),
        );
    }

    if commands.user_side == side {
        commands.push(UpdateType::Animation, delay(1500));
    }
}

fn keep_hand(commands: &mut ResponseBuilder, game: &GameState, side: Side, cards: &[CardId]) {
    for card_id in cards {
        commands.move_card(
            UpdateType::Animation,
            game.card(*card_id),
            Position::Hand(ObjectPositionHand { owner: commands.adapt_player_name(side) }),
        );
    }

    commands.apply_parallel_moves();

    for card_id in cards {
        // Need to manually update cards to change their 'can play' value
        commands.push(
            UpdateType::Animation,
            create_or_update(
                game,
                commands.user_side,
                *card_id,
                CardCreationStrategy::SnapToCurrentPosition,
            ),
        )
    }
}

fn mulligan_hand(
    commands: &mut ResponseBuilder,
    game: &GameState,
    side: Side,
    old_cards: &[CardId],
    new_cards: &[CardId],
) {
    let mulligan_player_name = adapters::to_player_name(side, commands.user_side);
    for card_id in old_cards {
        commands.move_card(
            UpdateType::Animation,
            game.card(*card_id),
            Position::Deck(ObjectPositionDeck { owner: mulligan_player_name.into() }),
        );
    }
    commands.apply_parallel_moves();

    for card_id in old_cards {
        destroy_card(commands, UpdateType::Animation, *card_id);
    }

    for card_id in new_cards {
        commands.push(
            UpdateType::Animation,
            create_or_update(
                game,
                commands.user_side,
                *card_id,
                CardCreationStrategy::CreateAtPosition(ObjectPosition {
                    sorting_key: game.card(*card_id).sorting_key,
                    position: Some(Position::Deck(ObjectPositionDeck {
                        owner: mulligan_player_name.into(),
                    })),
                }),
            ),
        );

        if side == commands.user_side {
            commands.move_card_immediate(
                UpdateType::Animation,
                game.card(*card_id),
                Position::Browser(ObjectPositionBrowser {}),
            );
        }
    }

    if side == commands.user_side {
        commands.push(UpdateType::Animation, delay(1500));
    }

    for card_id in new_cards {
        commands.move_card(
            UpdateType::Animation,
            game.card(*card_id),
            Position::Hand(ObjectPositionHand { owner: mulligan_player_name.into() }),
        );
    }

    commands.apply_parallel_moves();
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
        UpdateType::Utility,
        Command::CreateOrUpdateCard(full_sync::create_or_update_card(
            game,
            game.card(card_id),
            user_side,
            creation_strategy,
        )),
    );

    move_card(commands, UpdateType::Utility, game.card(card_id));
}

/// Appends a move card command to move a card to its current location. Skips
/// appending the command if the destination would not be a valid game position,
/// e.g. if it is [CardPosition::DeckUnknown].
fn move_card(commands: &mut ResponseBuilder, update_type: UpdateType, card: &CardState) {
    commands.move_object_optional(
        update_type,
        Id::CardId(adapters::adapt_card_id(card.id)),
        full_sync::adapt_position(card, commands.user_side),
    );
}

/// Commands to reveal the indicated card to all players
fn reveal_card(commands: &mut ResponseBuilder, game: &GameState, card: &CardState) {
    if commands.user_side != card.side() && game.data.raid.is_none() {
        // If there is no active raid, animate the card to the staging area on reveal.
        commands.push(
            UpdateType::Reveal,
            Command::MoveGameObjects(MoveGameObjectsCommand {
                ids: vec![adapters::card_id_to_object_id(card.id)],
                position: Some(ObjectPosition {
                    sorting_key: 0,
                    position: Some(Position::Staging(ObjectPositionStaging {})),
                }),
                disable_animation: false,
            }),
        );
        commands.push(
            UpdateType::Reveal,
            create_or_update(
                game,
                commands.user_side,
                card.id,
                CardCreationStrategy::SnapToCurrentPosition,
            ),
        );

        if let Some(position) = full_sync::adapt_position(card, commands.user_side) {
            commands.push(UpdateType::Reveal, delay(1500));
            commands.move_object_immediate(
                UpdateType::Reveal,
                Id::CardId(adapters::adapt_card_id(card.id)),
                position,
            );
        }
    }
}

/// Starts the `side` player's turn
fn start_turn(commands: &mut ResponseBuilder, side: Side) {
    commands.push(
        UpdateType::Animation,
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
            UpdateType::Animation,
            Command::VisitRoom(VisitRoomCommand {
                initiator: commands.adapt_player_name(Side::Champion),
                room_id: adapters::adapt_room_id(target).into(),
                visit_type: RoomVisitType::InitiateRaid.into(),
            }),
        );
        commands.push(UpdateType::Animation, delay(500));
    }

    commands.move_object(
        UpdateType::Animation,
        Id::Identity(commands.adapt_player_name(Side::Champion)),
        ObjectPosition {
            sorting_key: 0,
            position: Some(Position::IdentityContainer(ObjectPositionIdentityContainer {
                owner: commands.adapt_player_name(Side::Champion),
            })),
        },
    )
}

fn level_up_room(commands: &mut ResponseBuilder, target: RoomId) {
    let overlord_player_name = commands.adapt_player_name(Side::Overlord);
    if commands.user_side == Side::Champion {
        commands.push(
            UpdateType::Animation,
            Command::VisitRoom(VisitRoomCommand {
                initiator: overlord_player_name,
                room_id: adapters::adapt_room_id(target).into(),
                visit_type: RoomVisitType::LevelUpRoom.into(),
            }),
        );
        commands.push(UpdateType::Animation, delay(500));
    }

    commands.move_object(
        UpdateType::Animation,
        Id::Identity(overlord_player_name),
        ObjectPosition {
            sorting_key: 0,
            position: Some(Position::IdentityContainer(ObjectPositionIdentityContainer {
                owner: overlord_player_name,
            })),
        },
    )
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
    commands.push(UpdateType::Animation, Command::FireProjectile(projectile));
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

fn score_card(commands: &mut ResponseBuilder, game: &GameState, card: &CardState, side: Side) {
    let object_id = adapters::card_id_to_object_id(card.id);

    if side == Side::Overlord && commands.user_side == Side::Champion {
        commands.push(
            UpdateType::Animation,
            create_or_update(
                game,
                commands.user_side,
                card.id,
                CardCreationStrategy::SnapToCurrentPosition,
            ),
        );
    }

    commands.push(UpdateType::Animation, set_music(MusicState::Silent));
    commands.push(
        UpdateType::Animation,
        play_sound(SoundEffect::FantasyEvents(FantasyEventSounds::Positive1)),
    );
    commands.push(
        UpdateType::Animation,
        move_to(
            object_id,
            ObjectPosition {
                sorting_key: card.sorting_key,
                position: Some(Position::ScoreAnimation(ObjectPositionScoreAnimation {})),
            },
        ),
    );
    commands.push(
        UpdateType::Animation,
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
        UpdateType::Animation,
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

    commands.push(
        UpdateType::Animation,
        move_to(
            object_id,
            ObjectPosition {
                sorting_key: card.sorting_key,
                position: Some(Position::Identity(ObjectPositionIdentity {
                    owner: commands.adapt_player_name(side),
                })),
            },
        ),
    );
}

/// Moves a card to its owner's deck and then destroys it.
fn shuffle_into_deck(
    commands: &mut ResponseBuilder,
    game: &GameState,
    user_side: Side,
    card_id: CardId,
    update_type: UpdateType,
) {
    commands.move_card(
        update_type,
        game.card(card_id),
        Position::Deck(ObjectPositionDeck {
            owner: adapters::to_player_name(card_id.side, user_side).into(),
        }),
    );

    destroy_card(commands, update_type, card_id);
}

fn destroy_card(commands: &mut ResponseBuilder, update_type: UpdateType, card_id: CardId) {
    commands.push(
        update_type,
        Command::DestroyCard(DestroyCardCommand {
            card_id: Some(adapters::adapt_card_id(card_id)),
        }),
    );
}

/// Moves an ability card to its owner's hand, e.g. to return it after it has
/// been played.
fn move_ability_to_hand(commands: &mut ResponseBuilder, game: &GameState, ability_id: AbilityId) {
    let identifier = adapters::adapt_ability_id(ability_id);
    commands.push(
        UpdateType::Animation,
        Command::MoveGameObjects(MoveGameObjectsCommand {
            ids: vec![GameObjectIdentifier { id: Some(Id::CardId(identifier)) }],
            position: Some(ObjectPosition {
                sorting_key: game.card(ability_id.card_id).sorting_key,
                position: Some(Position::Hand(ObjectPositionHand {
                    owner: commands.adapt_player_name(ability_id.card_id.side),
                })),
            }),
            disable_animation: false,
        }),
    );
}

/// Animates a token card appearing, representing an ability being activated or
/// triggered.
fn show_ability_fired(commands: &mut ResponseBuilder, game: &GameState, ability_id: AbilityId) {
    let identifier = adapters::adapt_ability_id(ability_id);
    let card = full_sync::ability_card_view(
        game,
        ability_id,
        commands.user_side,
        false, /* check_can_play */
    );
    commands.push(
        UpdateType::Animation,
        Command::CreateOrUpdateCard(CreateOrUpdateCardCommand {
            card: Some(card),
            create_position: Some(ObjectPosition {
                sorting_key: game.card(ability_id.card_id).sorting_key,
                position: Some(Position::Staging(ObjectPositionStaging {})),
            }),
            create_animation: CardCreationAnimation::FromParentCard.into(),
            disable_flip_animation: true,
        }),
    );
    commands.push(UpdateType::Animation, delay(1500));

    commands.push(
        UpdateType::Animation,
        Command::MoveGameObjects(MoveGameObjectsCommand {
            ids: vec![GameObjectIdentifier { id: Some(Id::CardId(identifier)) }],
            position: Some(ObjectPosition {
                sorting_key: 0,
                position: Some(Position::IntoCard(ObjectPositionIntoCard {
                    card_id: Some(adapters::adapt_card_id(ability_id.card_id)),
                })),
            }),
            disable_animation: false,
        }),
    );

    commands.push(
        UpdateType::Animation,
        Command::DestroyCard(DestroyCardCommand { card_id: Some(identifier) }),
    );
}

fn game_over(commands: &mut ResponseBuilder, game: &GameState, winner: Side) {
    commands.push(UpdateType::Animation, delay(500));

    commands.push(
        UpdateType::Animation,
        Command::SetGameObjectsEnabled(SetGameObjectsEnabledCommand {
            game_objects_enabled: false,
        }),
    );

    commands.push(
        UpdateType::Animation,
        Command::DisplayGameMessage(DisplayGameMessageCommand {
            message_type: if winner == commands.user_side {
                GameMessageType::Victory
            } else {
                GameMessageType::Defeat
            }
            .into(),
        }),
    );

    if winner == commands.user_side {
        // TODO: Show real rewards instead of placeholder values
        commands.push(
            UpdateType::Animation,
            Command::DisplayRewards(DisplayRewardsCommand {
                rewards: game
                    .cards(winner)
                    .iter()
                    .filter(|card| card.is_revealed_to(winner) && !card.position().is_identity())
                    .take(5)
                    .map(|card| full_sync::card_view(game, card, winner))
                    .collect(),
            }),
        );
    }
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

fn create_or_update(
    game: &GameState,
    user_side: Side,
    card_id: CardId,
    creation: CardCreationStrategy,
) -> Command {
    Command::CreateOrUpdateCard(full_sync::create_or_update_card(
        game,
        game.card(card_id),
        user_side,
        creation,
    ))
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
