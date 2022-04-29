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

//! Contains functions for responding to user-initiated game actions received
//! from the client. The `handle_user_action` function is the primary
//! entry-point into the rules engine.
//!
//! By convention, functions in this module are responsible for validating the
//! legality of requests and returning [Result] accordingly. Beyond this point,
//! game functions typically assume the game is in a valid state and will panic
//! if that is not true.

use anyhow::{bail, ensure, Context, Result};
use data::card_definition::AbilityType;
use data::card_state::CardPosition;
use data::delegates::{ActivateAbilityEvent, CastCardEvent, PayCardCostsEvent};
use data::game::{GamePhase, GameState, MulliganDecision};
use data::game_actions::{CardTarget, PromptAction, UserAction};
use data::primitives::{AbilityId, CardId, CardType, ItemLocation, RoomId, RoomLocation, Side};
use data::updates::GameUpdate;
use tracing::{info, instrument};

use crate::raid_actions::initiate_raid_action;
use crate::{dispatch, flags, mutations, queries, raid_actions};

/// Top level dispatch function responsible for mutating [GameState] in response
/// to all [UserAction]s
pub fn handle_user_action(game: &mut GameState, user_side: Side, action: UserAction) -> Result<()> {
    match action {
        UserAction::Debug(_) => bail!("Rules engine does not handle debug actions!"),
        UserAction::PromptResponse(prompt_action) => {
            handle_prompt_action(game, user_side, prompt_action)
        }
        UserAction::GainMana => gain_mana_action(game, user_side),
        UserAction::DrawCard => draw_card_action(game, user_side),
        UserAction::PlayCard(card_id, target) => play_card_action(game, user_side, card_id, target),
        UserAction::ActivateAbility(ability_id, _target) => {
            // TODO: Handle ability targets
            activate_ability_action(game, user_side, ability_id)
        }
        UserAction::InitiateRaid(room_id) => initiate_raid_action(game, user_side, room_id),
        UserAction::LevelUpRoom(room_id) => level_up_room_action(game, user_side, room_id),
        UserAction::SpendActionPoint => spend_action_point_action(game, user_side),
    }
}

/// Handles a choice to keep or mulligan an opening hand
fn handle_mulligan_decision(
    game: &mut GameState,
    user_side: Side,
    decision: MulliganDecision,
) -> Result<()> {
    info!(?user_side, ?decision, "handle_mulligan_decision");
    ensure!(
        flags::can_make_mulligan_decision(game, user_side),
        "Cannot make mulligan decision for {:?}",
        user_side
    );
    let mut mulligans = match &mut game.data.phase {
        GamePhase::ResolveMulligans(mulligans) => mulligans,
        _ => bail!("Incorrect game phase"),
    };

    match user_side {
        Side::Overlord => mulligans.overlord = Some(decision),
        Side::Champion => mulligans.champion = Some(decision),
    }

    let hand = game.hand(user_side).map(|c| c.id).collect::<Vec<_>>();
    match decision {
        MulliganDecision::Keep => {
            game.updates.push(GameUpdate::KeepHand(user_side, hand));
        }
        MulliganDecision::Mulligan => {
            mutations::shuffle_into_deck(game, user_side, &hand);
            let new_hand = mutations::draw_cards(game, user_side, 5);
            game.updates.push(GameUpdate::MulliganHand(user_side, hand, new_hand));
        }
    }

    mutations::check_start_game(game);

    Ok(())
}

/// The basic game action to draw a card during your turn by spending one
/// action.
#[instrument(skip(game))]
fn draw_card_action(game: &mut GameState, user_side: Side) -> Result<()> {
    info!(?user_side, "draw_card_action");
    ensure!(
        flags::can_take_draw_card_action(game, user_side),
        "Cannot draw card for {:?}",
        user_side
    );
    mutations::spend_action_points(game, user_side, 1);
    mutations::draw_cards(game, user_side, 1);
    mutations::check_end_turn(game, user_side);
    Ok(())
}

/// The basic game action to play a card during your turn. Spends the resource
/// cost for a card, resolves its effects, and then moves it to the appropriate
/// new [CardPosition]. Spell, Weapon, and Artifact cards are immediately
/// revealed when played.
#[instrument(skip(game))]
fn play_card_action(
    game: &mut GameState,
    user_side: Side,
    card_id: CardId,
    target: CardTarget,
) -> Result<()> {
    info!(?user_side, ?card_id, ?target, "play_card_action");
    ensure!(
        flags::can_take_play_card_action(game, user_side, card_id, target),
        "Cannot play card {:?}",
        card_id
    );
    let card = game.card(card_id);
    let definition = crate::get(card.name);
    let enters_face_up = flags::enters_play_face_up(game, card_id);

    if enters_face_up {
        mutations::spend_mana(
            game,
            user_side,
            queries::mana_cost(game, card_id).with_context(|| "Card has no mana cost")?,
        );
    }

    mutations::spend_action_points(game, user_side, definition.cost.actions);
    dispatch::invoke_event(game, PayCardCostsEvent(card_id));
    dispatch::invoke_event(game, CastCardEvent(card_id));

    let new_position = match definition.card_type {
        CardType::Spell | CardType::Sorcery => CardPosition::DiscardPile(user_side),
        CardType::Weapon => CardPosition::ArenaItem(ItemLocation::Weapons),
        CardType::Artifact => CardPosition::ArenaItem(ItemLocation::Artifacts),
        CardType::Minion => CardPosition::Room(target.room_id()?, RoomLocation::Defender),
        CardType::Project | CardType::Scheme => {
            CardPosition::Room(target.room_id()?, RoomLocation::Occupant)
        }
        CardType::Identity => CardPosition::Identity(user_side),
    };

    if enters_face_up {
        mutations::turn_face_up(game, card_id);
    }

    mutations::move_card(game, card_id, new_position);

    mutations::check_end_turn(game, user_side);
    Ok(())
}

/// The basic game action to activate an ability of a card in play.
#[instrument(skip(game))]
fn activate_ability_action(
    game: &mut GameState,
    user_side: Side,
    ability_id: AbilityId,
) -> Result<()> {
    info!(?user_side, ?ability_id, "activate_ability_action");
    ensure!(
        flags::can_take_activate_ability_action(game, user_side, ability_id),
        "Cannot activate ability {:?}",
        ability_id
    );
    let card = game.card(ability_id.card_id);

    if let AbilityType::Activated(cost) =
        &crate::get(card.name).ability(ability_id.index).ability_type
    {
        mutations::spend_action_points(game, user_side, cost.actions);
        if let Some(mana) = queries::ability_mana_cost(game, ability_id) {
            mutations::spend_mana(game, user_side, mana);
        }
    }

    dispatch::invoke_event(game, ActivateAbilityEvent(ability_id));
    game.updates.push(GameUpdate::AbilityActivated(ability_id));
    mutations::check_end_turn(game, user_side);
    Ok(())
}

/// The basic game action to gain 1 mana during your turn by spending one
/// action.
#[instrument(skip(game))]
fn gain_mana_action(game: &mut GameState, user_side: Side) -> Result<()> {
    info!(?user_side, "gain_mana_action");
    ensure!(
        flags::can_take_gain_mana_action(game, user_side),
        "Cannot gain mana for {:?}",
        user_side
    );
    mutations::spend_action_points(game, user_side, 1);
    mutations::gain_mana(game, user_side, 1);
    mutations::check_end_turn(game, user_side);
    Ok(())
}

fn level_up_room_action(game: &mut GameState, user_side: Side, room_id: RoomId) -> Result<()> {
    info!(?user_side, "level_up_room_action");
    ensure!(
        flags::can_take_level_up_room_action(game, user_side, room_id),
        "Cannot level up room for {:?}",
        user_side
    );
    mutations::spend_action_points(game, user_side, 1);
    mutations::spend_mana(game, user_side, 1);
    mutations::level_up_room(game, room_id);

    mutations::check_end_turn(game, user_side);
    game.updates.push(GameUpdate::LevelUpRoom(room_id));

    Ok(())
}

fn spend_action_point_action(game: &mut GameState, user_side: Side) -> Result<()> {
    ensure!(
        queries::in_main_phase(game, user_side),
        "Cannot spend action point for {:?}",
        user_side
    );
    mutations::spend_action_points(game, user_side, 1);
    mutations::check_end_turn(game, user_side);
    Ok(())
}

/// Handles a [PromptAction] for the `user_side` player. Clears active prompts.
fn handle_prompt_action(game: &mut GameState, user_side: Side, action: PromptAction) -> Result<()> {
    ensure!(
        matches!(
            &game.player(user_side).prompt,
            Some(prompt) if prompt.responses.iter().any(|p| p == &action)
        ),
        "Unexpected action {:?} received",
        action
    );
    mutations::clear_prompt(game, user_side);

    match action {
        PromptAction::MulliganDecision(mulligan) => {
            handle_mulligan_decision(game, user_side, mulligan)
        }
        PromptAction::ActivateRoomAction(data) => {
            raid_actions::room_activation_action(game, user_side, data)
        }
        PromptAction::EncounterAction(data) => {
            raid_actions::encounter_action(game, user_side, data)
        }
        PromptAction::ContinueAction(data) => raid_actions::continue_action(game, user_side, data),
        PromptAction::RaidDestroyCard(card_id) => {
            raid_actions::destroy_card_action(game, user_side, card_id)
        }
        PromptAction::RaidScoreCard(card_id) => {
            raid_actions::score_card_action(game, user_side, card_id)
        }
        PromptAction::EndRaid => raid_actions::raid_end_action(game, user_side),
    }
}
