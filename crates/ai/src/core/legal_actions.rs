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

//! Identifies legal game actions for a given game state.

use std::iter;

use data::game::{GamePhase, GameState};
use data::game_actions::{CardTarget, CardTargetKind, UserAction};
use data::primitives::{CardId, RoomId, Side};
use enum_iterator::IntoEnumIterator;
use rules::{flags, queries};

/// Returns an iterator over currently-legal [UserAction]s for the `side` player
/// in the given [GameState].
pub fn evaluate<'a>(game: &'a GameState, side: Side) -> Box<dyn Iterator<Item = UserAction> + 'a> {
    if let GamePhase::GameOver(_) = &game.data.phase {
        return Box::new(iter::empty());
    }

    if let Some(prompt) = &game.player(side).prompt {
        return Box::new(prompt.responses.iter().map(|prompt| UserAction::PromptResponse(*prompt)));
    }

    if queries::in_main_phase(game, side) {
        Box::new(
            RoomId::into_enum_iter()
                .filter(move |room_id| flags::can_initiate_raid(game, side, *room_id))
                .map(UserAction::InitiateRaid)
                .chain(
                    RoomId::into_enum_iter()
                        .filter(move |room_id| flags::can_level_up_room(game, side, *room_id))
                        .map(UserAction::LevelUpRoom),
                )
                .chain(game.hand(side).flat_map(move |c| legal_card_actions(game, side, c.id)))
                .chain(iter::once(UserAction::GainMana))
                .chain(iter::once(UserAction::DrawCard)),
        )
    } else {
        Box::new(iter::empty())
    }
}

/// Builds an iterator over all possible 'play card' and 'activate ability'
/// actions for the provided card.
fn legal_card_actions(
    game: &GameState,
    side: Side,
    card_id: CardId,
) -> impl Iterator<Item = UserAction> + '_ {
    let target_kind = queries::card_target_kind(game, card_id);

    // Iterator combining pattern suggested by *the* Niko Matsakis
    // https://stackoverflow.com/a/52064434/298036
    let play_in_room = if target_kind == CardTargetKind::Room {
        Some(RoomId::into_enum_iter().filter_map(move |room_id| {
            if flags::can_take_play_card_action(game, side, card_id, CardTarget::Room(room_id)) {
                Some(UserAction::PlayCard(card_id, CardTarget::Room(room_id)))
            } else {
                None
            }
        }))
    } else {
        None
    };

    let play_card = if target_kind == CardTargetKind::None
        && flags::can_take_play_card_action(game, side, card_id, CardTarget::None)
    {
        Some(iter::once(UserAction::PlayCard(card_id, CardTarget::None)))
    } else {
        None
    };

    let activate_abilities =
        rules::card_definition(game, card_id).ability_ids(card_id).filter_map(move |ability_id| {
            if flags::can_take_activate_ability_action(game, side, ability_id) {
                // TODO: Handle targeted abilities
                Some(UserAction::ActivateAbility(ability_id, CardTarget::None))
            } else {
                None
            }
        });

    play_in_room
        .into_iter()
        .flatten()
        .chain(play_card.into_iter().flatten())
        .chain(activate_abilities.into_iter())
}
