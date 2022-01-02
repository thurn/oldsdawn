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

use data::card_state::CardPositionKind;
use data::game::GameState;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{
    ArenaView, CardIcon, CardIcons, CardView, CreateOrUpdateCardCommand, GameCommand, GameView,
    PlayerInfo, PlayerView, RevealedCardView, UpdateGameViewCommand,
};

use crate::full_sync::FullSync;

pub fn execute(
    commands: &mut Vec<GameCommand>,
    game: &GameState,
    old: Option<&FullSync>,
    new: &FullSync,
) {
    commands.push(GameCommand {
        command: diff_update_game_view_command(old.map(|old| &old.game), Some(&new.game))
            .map(Command::UpdateGameView),
    });
    commands.extend(
        // Iterate over `all_cards` again to ensure response order is deterministic
        game.all_cards().filter(|c| c.position.kind() != CardPositionKind::DeckUnknown).filter_map(
            |card| {
                diff_create_or_update_card(
                    old.and_then(|old| old.cards.get(&card.id)),
                    new.cards.get(&card.id),
                )
                .map(|command| GameCommand { command: Some(Command::CreateOrUpdateCard(command)) })
            },
        ),
    );
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
        arena: diff_arena_view(old.arena.as_ref(), new.arena.as_ref()),
        current_priority: new.current_priority,
    })
}

fn diff_player_view(old: Option<&PlayerView>, new: Option<&PlayerView>) -> Option<PlayerView> {
    run_diff(old, new, |old, new| PlayerView {
        player_info: diff_player_info(old.player_info.as_ref(), new.player_info.as_ref()),
        score: diff_simple(&old.score, &new.score),
        mana: diff_simple(&old.mana, &new.mana),
        action_tracker: diff_simple(&old.action_tracker, &new.action_tracker),
    })
}

fn diff_arena_view(old: Option<&ArenaView>, new: Option<&ArenaView>) -> Option<ArenaView> {
    run_diff(old, new, |old, new| ArenaView {
        rooms_at_bottom: diff_simple(&old.rooms_at_bottom, &new.rooms_at_bottom),
        identity_action: diff_simple(&old.identity_action, &new.identity_action),
    })
}

fn diff_player_info(old: Option<&PlayerInfo>, new: Option<&PlayerInfo>) -> Option<PlayerInfo> {
    run_diff(old, new, |old, new| PlayerInfo {
        name: diff_simple(&old.name, &new.name),
        portrait: diff_simple(&old.portrait, &new.portrait),
        portrait_frame: diff_simple(&old.portrait_frame, &new.portrait_frame),
        card_back: diff_simple(&old.card_back, &new.card_back),
    })
}

fn diff_create_or_update_card(
    old: Option<&CreateOrUpdateCardCommand>,
    new: Option<&CreateOrUpdateCardCommand>,
) -> Option<CreateOrUpdateCardCommand> {
    // We only want to send this command if the card's own state has changed.
    // Changes to create behavior should be handled by the animation layer.
    let card_view = diff_card_view(
        old.and_then(|old| old.card.as_ref()),
        new.and_then(|new| new.card.as_ref()),
    );

    if let (Some(card_view), Some(new)) = (card_view, new) {
        Some(CreateOrUpdateCardCommand {
            card: Some(card_view),
            create_position: new.create_position.clone(),
            create_animation: new.create_animation,
            disable_flip_animation: new.disable_flip_animation,
        })
    } else {
        None
    }
}

fn diff_card_view(old: Option<&CardView>, new: Option<&CardView>) -> Option<CardView> {
    run_diff(old, new, |old, new| CardView {
        card_id: new.card_id.clone(),
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
        revealed_in_arena: new.revealed_in_arena,
        targeting: diff_simple(&old.targeting, &new.targeting),
        on_release_position: diff_simple(&old.on_release_position, &new.on_release_position),
        can_play: new.can_play,
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
        background: diff_simple(&old.background, &new.background),
        text: diff_simple(&old.text, &new.text),
        background_scale: new.background_scale,
    })
}

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
