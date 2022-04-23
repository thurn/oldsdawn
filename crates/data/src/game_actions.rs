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
pub enum RoomActivationAction {
    Activate,
    Pass,
}
#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum EncounterAction {
    /// (source_id, target_id)
    UseWeaponAbility(CardId, CardId),
    NoWeapon,
}

#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum ContinueAction {
    Advance,
    Retreat,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum PromptContext {
    ActivateRoom,
    RaidAdvance,
}

/// An action which can be taken in the user interface, typically embedded
/// inside the `GameAction::StandardAction` protobuf message type when sent to
/// the client.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum PromptAction {
    /// Action to keep or mulligan opening hand
    MulliganDecision(MulliganDecision),
    /// Action for the Overlord to activate the room currently being raided
    ActivateRoomAction(RoomActivationAction),
    /// Champion action in response to a raid encounter
    EncounterAction(EncounterAction),
    /// Action to advance to the next encounter of a raid or retreat
    ContinueAction(ContinueAction),
    /// Action to target & destroy an accessed card
    RaidDestroyCard(CardId),
    /// Action to score an accessed card
    RaidScoreCard(CardId),
    /// Action to end a raid after the access phase
    EndRaid,
}

/// Presents a choice to a user, typically communicated via a series of buttons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    /// Identifies the context for this prompt, i.e. why it is being shown to
    /// the user
    pub context: Option<PromptContext>,
    /// Possible responses to this prompt
    pub responses: Vec<PromptAction>,
}

/// Actions that can be taken from the debug panel, should not be exposed in
/// production.
#[derive(Eq, PartialEq, Hash, Debug, Copy, Clone, Serialize, Deserialize)]
pub enum DebugAction {
    NewGame(Side),
    JoinGame,
    ResetGame,
    FetchStandardPanels,
    AddMana(ManaValue),
    AddActionPoints(ActionCount),
    AddScore(PointsValue),
    SwitchTurn,
    FlipViewpoint,
    SaveState(u64),
    LoadState(u64),
    SetAgent(Side, GameStatePredictorName, AgentName),
}

/// Possible targets for the 'play card' action. Note that many types of targets
/// are *not* selected in the original PlayCard action request but are instead
/// selected via a follow-up prompt, and thus are not represented here.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Serialize, Deserialize, EnumKind)]
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
    PromptResponse(PromptAction),
    GainMana,
    DrawCard,
    PlayCard(CardId, CardTarget),
    ActivateAbility(AbilityId, CardTarget),
    InitiateRaid(RoomId),
    LevelUpRoom(RoomId),
    SpendActionPoint,
}
