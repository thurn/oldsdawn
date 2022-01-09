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

use enum_kinds::EnumKind;
use serde::{Deserialize, Serialize};

use crate::primitives::CardId;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum RaidActivateRoom {
    Activate,
    Pass,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum RaidEncounter {
    UseWeaponAbility(CardId),
    Continue,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum RaidAdvance {
    Continue,
    Retreat,
}

/// An action which can be taken in the user interface, typically embedded
/// inside the `GameAction::StandardAction` protobuf message type when sent to
/// the client.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, EnumKind)]
#[enum_kind(PromptKind, derive(Serialize), derive(Deserialize))]
pub enum PromptResponse {
    /// Action for the Overlord to activate the room currently being raided
    RaidActivateRoom(RaidActivateRoom),
    /// Champion action in response to a raid encounter
    RaidEncounter(RaidEncounter),
    /// Action to advance to the next encounter of a raid
    RaidAdvance(RaidAdvance),
    /// Action to target & destroy an accessed card
    RaidDestroyCard(CardId),
    /// Action to score an accessed card
    RaidScoreCard(CardId),
    /// Action to end a raid after the access phase
    RaidEnd,
}

impl PromptResponse {
    pub fn kind(&self) -> PromptKind {
        self.into()
    }
}

/// Presents a choice to a user, typically communicated via a series of buttons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    /// Identifies the context for this prompt, i.e. why it is being shown to
    /// the user
    pub kind: PromptKind,
    /// Possible responses to this prompt
    pub responses: Vec<PromptResponse>,
}
