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

//! User interface actions

#![allow(clippy::use_self)] // Required to use EnumKind

use anyhow::{anyhow, Result};
use enum_kinds::EnumKind;
use serde::{Deserialize, Serialize};

use crate::agent_definition::{AgentName, GameStatePredictorName};
use crate::game::MulliganDecision;
use crate::primitives::{AbilityId, ActionCount, CardId, ManaValue, PointsValue, RoomId, Side};

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum EncounterAction {
    /// (source_id, target_id)
    UseWeaponAbility(CardId, CardId),
    NoWeapon,
    /// Custom card action, resolved and then treated equivalently to 'no
    /// weapon'
    CardAction(CardPromptAction),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum AccessPhaseAction {
    ScoreCard(CardId),
    DestroyCard(CardId, ManaValue),
    EndRaid,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PromptContext {
    RaidAdvance,
}

/// A choice which can be made as part of an ability of an individual card
///
/// Maybe switch this to a trait someday?
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum CardPromptAction {
    /// A player loses mana
    LoseMana(Side, ManaValue),
    /// A player loses action points
    LoseActions(Side, ActionCount),
    /// End the current raid in failure.
    EndRaid,
    /// Deal damage to the Champion
    TakeDamage(AbilityId, u32),
    /// Deal damage and end the current raid
    TakeDamageEndRaid(AbilityId, u32),
}

/// An action which can be taken in the user interface, typically embedded
/// inside the `GameAction::StandardAction` protobuf message type when sent to
/// the client.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum PromptAction {
    /// Action to keep or mulligan opening hand
    MulliganDecision(MulliganDecision),
    /// Champion action in response to a raid encounter
    EncounterAction(EncounterAction),
    /// Action to target & destroy an accessed card
    AccessPhaseAction(AccessPhaseAction),
    /// Action to take as part of a card ability
    CardAction(CardPromptAction),
}

/// Presents a choice to a user, typically communicated via a series of buttons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GamePrompt {
    /// Identifies the context for this prompt, i.e. why it is being shown to
    /// the user
    pub context: Option<PromptContext>,
    /// Possible responses to this prompt
    pub responses: Vec<PromptAction>,
}

impl GamePrompt {
    pub fn card_actions(actions: Vec<CardPromptAction>) -> Self {
        Self {
            context: None,
            responses: actions.into_iter().map(PromptAction::CardAction).collect(),
        }
    }
}

/// Actions that can be taken from the debug panel, should not be exposed in
/// production.
#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum DebugAction {
    // Creates a new game with ID 0, using the canonical decklist for [Side], playing against an
    // opponent who will take no actions. Overwrites the current player's player data with the
    // canonical decklists.
    NewGame(Side),

    // Adds the current player to the game with ID 0, overwriting the non-human player in this
    // game. Overwrites the current player's player data with the canonical decklists.
    JoinGame,

    // Swaps which side the current player is playing as in their current game.
    FlipViewpoint,

    AddMana(ManaValue),
    AddActionPoints(ActionCount),
    AddScore(PointsValue),
    SaveState(u64),
    LoadState(u64),
    SetAgent(Side, GameStatePredictorName, AgentName),
}

/// Possible targets for the 'play card' action. Note that many types of targets
/// are *not* selected in the original PlayCard action request but are instead
/// selected via a follow-up prompt, and thus are not represented here.
#[derive(
    PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize, EnumKind, Ord, PartialOrd,
)]
#[enum_kind(CardTargetKind)]
pub enum CardTarget {
    None,
    Room(RoomId),
}

impl CardTarget {
    /// Gets the RoomId targeted by a player, or returns an error if no target
    /// was provided.
    pub fn room_id(&self) -> Result<RoomId> {
        match self {
            CardTarget::Room(room_id) => Ok(*room_id),
            _ => Err(anyhow!("Expected a RoomId to be provided but got {:?}", self)),
        }
    }
}

/// All possible actions a player can take during a game.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum UserAction {
    Debug(DebugAction),
    PromptAction(PromptAction),
    GainMana,
    DrawCard,
    PlayCard(CardId, CardTarget),
    ActivateAbility(AbilityId, CardTarget),
    InitiateRaid(RoomId),
    LevelUpRoom(RoomId),
    SpendActionPoint,
}
