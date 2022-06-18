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
//! legality of requests and returning [Result] accordingly.

use anyhow::{bail, ensure, Result};
use data::card_definition::AbilityType;
use data::card_state::CardPosition;
use data::delegates::{
    AbilityActivated, ActivateAbilityEvent, CardPlayed, CastCardEvent, DrawCardActionEvent,
    PayCardCostsEvent,
};
use data::game::{GamePhase, GameState, MulliganDecision};
use data::game_actions::{CardTarget, GamePrompt, PromptAction, UserAction};
use data::primitives::{AbilityId, CardId, CardType, ItemLocation, RoomId, RoomLocation, Side};
use data::updates2::GameUpdate2;
use data::with_error::WithError;
use data::{fail, verify};
use tracing::{info, instrument};

use crate::card_prompt::HandleCardPrompt;
use crate::mana::ManaPurpose;
use crate::raid_actions::initiate_raid_action;
use crate::{card_prompt, dispatch, flags, mana, mutations, queries, raid_actions};

/// Top level dispatch function responsible for mutating [GameState] in response
/// to all [UserAction]s
pub fn handle_user_action(game: &mut GameState, user_side: Side, action: UserAction) -> Result<()> {
    match action {
        UserAction::Debug(_) => fail!("Rules engine does not handle debug actions!"),
        UserAction::GamePromptResponse(prompt_action) => {
            handle_prompt_action(game, user_side, prompt_action)
        }
        UserAction::GainMana => gain_mana_action(game, user_side),
        UserAction::DrawCard => draw_card_action(game, user_side),
        UserAction::PlayCard(card_id, target) => play_card_action(game, user_side, card_id, target),
        UserAction::ActivateAbility(ability_id, target) => {
            activate_ability_action(game, user_side, ability_id, target)
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
    verify!(
        flags::can_make_mulligan_decision(game, user_side),
        "Cannot make mulligan decision for {:?}",
        user_side
    );
    let mut mulligans = match &mut game.data.phase {
        GamePhase::ResolveMulligans(mulligans) => mulligans,
        _ => fail!("Incorrect game phase"),
    };

    match user_side {
        Side::Overlord => mulligans.overlord = Some(decision),
        Side::Champion => mulligans.champion = Some(decision),
    }

    let hand = game.hand(user_side).map(|c| c.id).collect::<Vec<_>>();
    match decision {
        MulliganDecision::Keep => {
            game.updates2.push(GameUpdate2::KeepHand(user_side, hand));
        }
        MulliganDecision::Mulligan => {
            mutations::shuffle_into_deck(game, user_side, &hand)?;
            let new_hand = mutations::draw_cards(game, user_side, 5)?;
            game.updates2.push(GameUpdate2::MulliganHand(user_side, hand, new_hand));
        }
    }

    mutations::check_start_game(game)?;

    Ok(())
}

/// The basic game action to draw a card during your turn by spending one
/// action.
#[instrument(skip(game))]
fn draw_card_action(game: &mut GameState, user_side: Side) -> Result<()> {
    info!(?user_side, "draw_card_action");
    verify!(
        flags::can_take_draw_card_action(game, user_side),
        "Cannot draw card for {:?}",
        user_side
    );
    mutations::spend_action_points(game, user_side, 1)?;
    let cards = mutations::draw_cards(game, user_side, 1)?;
    if let Some(card_id) = cards.get(0) {
        dispatch::invoke_event(game, DrawCardActionEvent(*card_id))?;
    }

    mutations::check_end_turn(game)?;
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
    verify!(
        flags::can_take_play_card_action(game, user_side, card_id, target),
        "Cannot play card {:?}",
        card_id
    );
    let card = game.card(card_id);
    let definition = crate::get(card.name);
    let enters_face_up = flags::enters_play_face_up(game, card_id);

    if enters_face_up {
        let amount = queries::mana_cost(game, card_id).with_error(|| "Card has no mana cost")?;
        mana::spend(game, user_side, ManaPurpose::PayForCard(card_id), amount)?;
        if let Some(custom_cost) = &definition.cost.custom_cost {
            (custom_cost.pay)(game, card_id)?;
        }
    }

    mutations::spend_action_points(game, user_side, definition.cost.actions)?;
    dispatch::invoke_event(game, PayCardCostsEvent(card_id))?;
    dispatch::invoke_event(game, CastCardEvent(CardPlayed { card_id, target }))?;

    let new_position = match definition.card_type {
        CardType::ChampionSpell | CardType::OverlordSpell => CardPosition::DiscardPile(user_side),
        CardType::Weapon => CardPosition::ArenaItem(ItemLocation::Weapons),
        CardType::Artifact => CardPosition::ArenaItem(ItemLocation::Artifacts),
        CardType::Minion => CardPosition::Room(target.room_id()?, RoomLocation::Defender),
        CardType::Project | CardType::Scheme => {
            CardPosition::Room(target.room_id()?, RoomLocation::Occupant)
        }
        CardType::Identity => CardPosition::Identity(user_side),
    };

    if enters_face_up {
        mutations::turn_face_up(game, card_id)?;
    }

    mutations::move_card(game, card_id, new_position)?;

    mutations::check_end_turn(game)?;
    Ok(())
}

/// The basic game action to activate an ability of a card in play.
#[instrument(skip(game))]
fn activate_ability_action(
    game: &mut GameState,
    user_side: Side,
    ability_id: AbilityId,
    target: CardTarget,
) -> Result<()> {
    info!(?user_side, ?ability_id, "activate_ability_action");
    verify!(
        flags::can_take_activate_ability_action(game, user_side, ability_id, target),
        "Cannot activate ability {:?}",
        ability_id
    );
    let card = game.card(ability_id.card_id);

    if let AbilityType::Activated(cost, _) =
        &crate::get(card.name).ability(ability_id.index).ability_type
    {
        mutations::spend_action_points(game, user_side, cost.actions)?;
        if let Some(mana) = queries::ability_mana_cost(game, ability_id) {
            mana::spend(game, user_side, ManaPurpose::ActivateAbility(ability_id), mana)?;
        }

        if let Some(custom_cost) = &cost.custom_cost {
            (custom_cost.pay)(game, ability_id)?;
        }
    } else {
        fail!("Ability is not an activated ability");
    }

    dispatch::invoke_event(game, ActivateAbilityEvent(AbilityActivated { ability_id, target }))?;
    game.updates2.push(GameUpdate2::AbilityActivated(ability_id));
    mutations::check_end_turn(game)?;
    Ok(())
}

/// The basic game action to gain 1 mana during your turn by spending one
/// action.
#[instrument(skip(game))]
fn gain_mana_action(game: &mut GameState, user_side: Side) -> Result<()> {
    info!(?user_side, "gain_mana_action");
    verify!(
        flags::can_take_gain_mana_action(game, user_side),
        "Cannot gain mana for {:?}",
        user_side
    );
    mutations::spend_action_points(game, user_side, 1)?;
    mana::gain(game, user_side, 1);
    mutations::check_end_turn(game)?;
    Ok(())
}

fn level_up_room_action(game: &mut GameState, user_side: Side, room_id: RoomId) -> Result<()> {
    info!(?user_side, "level_up_room_action");
    verify!(
        flags::can_take_level_up_room_action(game, user_side, room_id),
        "Cannot level up room for {:?}",
        user_side
    );
    mutations::spend_action_points(game, user_side, 1)?;
    mana::spend(game, user_side, ManaPurpose::LevelUpRoom(room_id), 1)?;
    mutations::level_up_room(game, room_id)?;

    mutations::check_end_turn(game)?;
    game.updates2.push(GameUpdate2::LevelUpRoom(room_id));

    Ok(())
}

fn spend_action_point_action(game: &mut GameState, user_side: Side) -> Result<()> {
    verify!(
        queries::in_main_phase(game, user_side),
        "Cannot spend action point for {:?}",
        user_side
    );
    mutations::spend_action_points(game, user_side, 1)?;
    mutations::check_end_turn(game)?;
    Ok(())
}

/// Handles a [PromptAction] for the `user_side` player. Clears active prompts.
fn handle_prompt_action(game: &mut GameState, user_side: Side, action: PromptAction) -> Result<()> {
    fn validate(prompt: &GamePrompt, action: &PromptAction) -> Result<()> {
        verify!(
            prompt.responses.iter().any(|p| p == action),
            "Unexpected action {:?} received",
            action
        );
        Ok(())
    }

    if let Some(prompt) = &game.player(user_side).card_prompt {
        validate(prompt, &action)?;
        game.player_mut(user_side).card_prompt = None;
    } else if let Some(prompt) = &game.player(user_side).game_prompt {
        validate(prompt, &action)?;
        game.player_mut(user_side).game_prompt = None;
    } else {
        fail!("Not expecting a prompt response");
    }

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
        PromptAction::CardAction(card_action) => {
            card_prompt::handle(game, user_side, card_action, HandleCardPrompt::ResetRaidPrompt)
        }
    }
}
