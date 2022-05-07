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

//! Functions for producing a diff between two game updates

use std::collections::BTreeMap;

use data::game::GameState;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::game_object_identifier::Id;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::{
    CardIcon, CardIcons, CardView, CreateOrUpdateCardCommand, GameView, ObjectPosition,
    ObjectPositionDeckContainer, ObjectPositionDiscardPileContainer,
    ObjectPositionIdentityContainer, PlayerInfo, PlayerName, PlayerView, RevealedCardView,
    UpdateGameViewCommand,
};

use crate::full_sync::FullSync;
use crate::response_builder::{ResponseBuilder, UpdateType};
use crate::{adapters, full_sync};

/// Performs a diff operation on two provided [FullSync] values, appending the
/// resulting commands to `commands`.
///
/// The diff algorithm examines the [FullSync] values for the game and for each
/// card in the game. For places where `old` and `new` differ, a command is
/// produced which contains the updated fields. Commands or fields which have
/// not changed are not updated, the client is assumed to preserve their state
/// between requests.
pub fn execute(
    commands: &mut ResponseBuilder,
    game: &GameState,
    old: Option<&FullSync>,
    new: &FullSync,
) {
    commands.apply_parallel_moves();

    if let Some(update) = diff_update_game_view_command(old.map(|old| &old.game), Some(&new.game)) {
        commands.push(UpdateType::General, Command::UpdateGameView(update));
    }

    // Iterate over `all_cards` again to ensure response order is deterministic
    for card_id in new.cards.keys() {
        diff_create_or_update_card(
            commands,
            old.and_then(|old| old.cards.get(card_id)),
            new.cards.get(card_id),
        );
    }

    diff_card_position_updates(
        commands,
        game,
        old.map(|old| &old.position_overrides),
        &new.position_overrides,
    );

    commands.apply_parallel_moves();

    if old.map(|old| &old.interface) != Some(&new.interface) {
        commands.push(UpdateType::General, Command::RenderInterface(new.interface.clone()));
    }
}

fn diff_update_game_view_command(
    old: Option<&UpdateGameViewCommand>,
    new: Option<&UpdateGameViewCommand>,
) -> Option<UpdateGameViewCommand> {
    run_diff(old, new, |old, new| UpdateGameViewCommand {
        game: diff_game_view(old.game.as_ref(), new.game.as_ref()),
    })
}

fn diff_game_view(old: Option<&GameView>, new: Option<&GameView>) -> Option<GameView> {
    run_diff(old, new, |old, new| GameView {
        game_id: new.game_id.clone(),
        user: diff_player_view(old.user.as_ref(), new.user.as_ref()),
        opponent: diff_player_view(old.opponent.as_ref(), new.opponent.as_ref()),
        raid_active: new.raid_active,
    })
}

fn diff_player_view(old: Option<&PlayerView>, new: Option<&PlayerView>) -> Option<PlayerView> {
    run_diff(old, new, |old, new| PlayerView {
        side: new.side,
        player_info: diff_player_info(old.player_info.as_ref(), new.player_info.as_ref()),
        score: diff_simple(&old.score, &new.score),
        mana: diff_simple(&old.mana, &new.mana),
        action_tracker: diff_simple(&old.action_tracker, &new.action_tracker),
        can_take_action: new.can_take_action,
    })
}

fn diff_player_info(old: Option<&PlayerInfo>, new: Option<&PlayerInfo>) -> Option<PlayerInfo> {
    run_diff(old, new, |old, new| PlayerInfo {
        name: diff_simple(&old.name, &new.name),
        portrait: diff_simple(&old.portrait, &new.portrait),
        portrait_frame: diff_simple(&old.portrait_frame, &new.portrait_frame),
        valid_rooms_to_visit: new.valid_rooms_to_visit.clone(),
        card_back: diff_simple(&old.card_back, &new.card_back),
    })
}

fn diff_create_or_update_card(
    commands: &mut ResponseBuilder,
    old: Option<&CreateOrUpdateCardCommand>,
    new: Option<&CreateOrUpdateCardCommand>,
) {
    // We only want to send this command if the card's own state has changed.
    // Changes to create behavior should be handled by the animation layer.
    let card_view = diff_card_view(
        old.and_then(|old| old.card.as_ref()),
        new.and_then(|new| new.card.as_ref()),
    );

    if let (Some(card_view), Some(new)) = (card_view, new) {
        commands.push(
            UpdateType::General,
            Command::CreateOrUpdateCard(CreateOrUpdateCardCommand {
                card: Some(card_view),
                create_position: new.create_position.clone(),
                create_animation: new.create_animation,
                disable_flip_animation: new.disable_flip_animation,
            }),
        );
    }
}

fn diff_card_view(old: Option<&CardView>, new: Option<&CardView>) -> Option<CardView> {
    run_diff(old, new, |old, new| CardView {
        card_id: new.card_id,
        prefab: new.prefab,
        revealed_to_viewer: new.revealed_to_viewer,
        is_face_up: new.is_face_up,
        card_icons: diff_card_icons(old.card_icons.as_ref(), new.card_icons.as_ref()),
        arena_frame: diff_simple(&old.arena_frame, &new.arena_frame),
        owning_player: new.owning_player,
        revealed_card: diff_revealed_card_view(
            old.revealed_card.as_ref(),
            new.revealed_card.as_ref(),
        ),
    })
}

fn diff_revealed_card_view(
    old: Option<&RevealedCardView>,
    new: Option<&RevealedCardView>,
) -> Option<RevealedCardView> {
    run_diff(old, new, |old, new| RevealedCardView {
        card_frame: diff_simple(&old.card_frame, &new.card_frame),
        title_background: diff_simple(&old.title_background, &new.title_background),
        jewel: diff_simple(&old.jewel, &new.jewel),
        image: diff_simple(&old.image, &new.image),
        title: diff_simple(&old.title, &new.title),
        rules_text: diff_simple(&old.rules_text, &new.rules_text),
        targeting: diff_simple(&old.targeting, &new.targeting),
        on_release_position: diff_simple(&old.on_release_position, &new.on_release_position),
        supplemental_info: diff_simple(&old.supplemental_info, &new.supplemental_info),
    })
}

fn diff_card_icons(old: Option<&CardIcons>, new: Option<&CardIcons>) -> Option<CardIcons> {
    run_diff(old, new, |old, new| CardIcons {
        top_left_icon: diff_card_icon(old.top_left_icon.as_ref(), new.top_left_icon.as_ref()),
        top_right_icon: diff_card_icon(old.top_right_icon.as_ref(), new.top_right_icon.as_ref()),
        bottom_right_icon: diff_card_icon(
            old.bottom_right_icon.as_ref(),
            new.bottom_right_icon.as_ref(),
        ),
        bottom_left_icon: diff_card_icon(
            old.bottom_left_icon.as_ref(),
            new.bottom_left_icon.as_ref(),
        ),
        arena_icon: diff_card_icon(old.arena_icon.as_ref(), new.arena_icon.as_ref()),
    })
}

fn diff_card_icon(old: Option<&CardIcon>, new: Option<&CardIcon>) -> Option<CardIcon> {
    run_diff(old, new, |old, new| CardIcon {
        enabled: new.enabled,
        background: diff_simple(&old.background, &new.background),
        text: diff_simple(&old.text, &new.text),
        background_scale: new.background_scale,
    })
}

fn diff_card_position_updates(
    commands: &mut ResponseBuilder,
    game: &GameState,
    old: Option<&BTreeMap<Id, ObjectPosition>>,
    new: &BTreeMap<Id, ObjectPosition>,
) {
    let mut ids = vec![
        Id::Identity(PlayerName::User.into()),
        Id::Identity(PlayerName::Opponent.into()),
        Id::Deck(PlayerName::User.into()),
        Id::Deck(PlayerName::Opponent.into()),
        Id::DiscardPile(PlayerName::User.into()),
        Id::DiscardPile(PlayerName::Opponent.into()),
    ];
    ids.extend(game.all_card_ids().map(|id| Id::CardId(adapters::adapt_card_id(id))));

    for id in ids {
        push_move_command(commands, game, old, new, id);
    }
}

/// Appends a command to update the position for the provided `id` if it has
/// changed between the position maps in `old` and `new`.
fn push_move_command(
    commands: &mut ResponseBuilder,
    game: &GameState,
    old: Option<&BTreeMap<Id, ObjectPosition>>,
    new: &BTreeMap<Id, ObjectPosition>,
    id: Id,
) {
    match old {
        None if new.contains_key(&id) => move_to_position(commands, game, id, new.get(&id)),
        Some(old) if old.get(&id) != new.get(&id) => {
            move_to_position(commands, game, id, new.get(&id))
        }
        _ => {}
    }
}

/// Appends a command to move `id` to its indicated `position` (if provided) or
/// else to its default game position.
fn move_to_position(
    commands: &mut ResponseBuilder,
    game: &GameState,
    id: Id,
    position: Option<&ObjectPosition>,
) {
    let new_position = if let Some(new) = position {
        new.clone()
    } else {
        match id {
            Id::CardId(card_id) => {
                let id = adapters::to_server_card_id(Some(card_id))
                    .expect("id")
                    .as_card_id()
                    .expect("card_id");
                if let Some(card_position) =
                    full_sync::adapt_position(game.card(id), commands.user_side)
                {
                    card_position
                } else {
                    // Card has no position, e.g. because it has been shuffled back into the deck.
                    // The effect which causes this transition is responsible for destroying it.
                    return;
                }
            }
            Id::Identity(name) => ObjectPosition {
                sorting_key: 0,
                position: Some(Position::IdentityContainer(ObjectPositionIdentityContainer {
                    owner: name,
                })),
            },
            Id::Deck(name) => ObjectPosition {
                sorting_key: 0,
                position: Some(Position::DeckContainer(ObjectPositionDeckContainer {
                    owner: name,
                })),
            },
            Id::DiscardPile(name) => ObjectPosition {
                sorting_key: 0,
                position: Some(Position::DiscardPileContainer(
                    ObjectPositionDiscardPileContainer { owner: name },
                )),
            },
        }
    };

    commands.move_object(UpdateType::General, id, new_position);
}

/// Diffs two values. If the values are equal, returns None, otherwise invokes
/// the provided `diff` function to produce some result.
fn run_diff<T: Clone + PartialEq>(
    old: Option<&T>,
    new: Option<&T>,
    diff: impl Fn(&T, &T) -> T,
) -> Option<T> {
    match (old, new) {
        (_, None) => None,
        (None, Some(new)) => Some(new.clone()),
        (Some(old), Some(new)) if old == new => None,
        (Some(old), Some(new)) => Some(diff(old, new)),
    }
}

/// Simplified version of [run_diff] which always clones the `new` value if the
/// two inputs are not equal.
fn diff_simple<T: Clone + PartialEq>(old: &Option<T>, new: &Option<T>) -> Option<T> {
    if old == new {
        None
    } else {
        new.clone()
    }
}
