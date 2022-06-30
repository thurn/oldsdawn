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

use anyhow::Result;
use data::game::{GameState, InternalRaidPhase};
use data::game_actions::{PromptAction, PromptContext};
use data::primitives::{CardId, Side};
use data::utils;
use fallible_iterator::FallibleIterator;

/// Represents how the current state of a raid should be represented in the user
/// interface -- with no content, as a sequence of defenders, or by showing
/// accessed cards.
pub enum RaidDisplayState {
    None,
    Defenders(Vec<CardId>),
    Access,
}

/// Primary trait for nodes in the Raid state machine.
///
/// Each state machine node corresponds to the `internal_phase` of a raid.
/// Typically external code should interact with these methods instead of
/// inspecting the internal raid phase itself, since raid logic may change.
pub trait RaidPhase {
    /// Invoked whenever the state machine enters this phase. The implementation
    /// may return a new [InternalRaidPhase] to immediately transition to, if no
    /// action is required.
    fn enter(&self, game: &mut GameState) -> Result<Option<InternalRaidPhase>>;

    /// Identifies the player who can currently act in the current phase.
    fn active_side(&self) -> Side;

    /// Describes how the current phase should be represented in the UI.
    fn display_state(&self, game: &GameState) -> Result<RaidDisplayState>;

    /// Provides UI context describing why a choice is being presented in the
    /// current phase.
    fn prompt_context(&self) -> Option<PromptContext>;

    /// Handles a user action in the current phase. This provided action is
    /// matched against the possible actions returned by the `prompts`
    /// function before invoking this method. May return a new
    /// [InternalRaidPhase] to transition the state machine to a new phase.
    fn handle_prompt(
        &self,
        game: &mut GameState,
        action: PromptAction,
    ) -> Result<Option<InternalRaidPhase>>;

    /// Provides a list of possible user actions for the `active_side` player in
    /// the current phase.
    fn prompts(&self, game: &GameState) -> Result<Vec<PromptAction>>;
}

/// Strongly-typed implementation trait for [RaidPhase] which specified the type
/// of game actions this phase operates on. This trait should be used when
/// implementing phases, but should generally not be invoked by calling code.
/// All structs which implement this struct also implement [RaidPhase] via
/// blanket implementation.
pub trait RaidPhaseImpl: RaidPhase + Sized + Copy {
    type Action;

    /// Convert a [PromptAction] into this phase's action type.
    fn unwrap(action: PromptAction) -> Result<Self::Action>;

    /// Convert this phase's action type in a [PromptAction].
    fn wrap(action: Self::Action) -> Result<PromptAction>;

    fn enter(self, game: &mut GameState) -> Result<Option<InternalRaidPhase>>;

    /// Strongly-typed equivalent of `prompts`.
    fn actions(self, game: &GameState) -> Result<Vec<Self::Action>>;

    /// Strongly-typed equivalent of `handle_prompt`.
    fn handle_action(
        self,
        game: &mut GameState,
        action: Self::Action,
    ) -> Result<Option<InternalRaidPhase>>;

    fn active_side(self) -> Side;

    fn display_state(self, game: &GameState) -> Result<RaidDisplayState>;

    fn prompt_context(self) -> Option<PromptContext> {
        None
    }

    fn handle_prompt(
        self,
        game: &mut GameState,
        action: PromptAction,
    ) -> Result<Option<InternalRaidPhase>> {
        self.handle_action(game, Self::unwrap(action)?)
    }

    fn prompts(self, game: &GameState) -> Result<Vec<PromptAction>> {
        utils::fallible(self.actions(game)?.into_iter()).map(Self::wrap).collect()
    }
}

impl<T: RaidPhaseImpl> RaidPhase for T {
    fn enter(&self, game: &mut GameState) -> Result<Option<InternalRaidPhase>> {
        RaidPhaseImpl::enter(*self, game)
    }

    fn active_side(&self) -> Side {
        RaidPhaseImpl::active_side(*self)
    }

    fn display_state(&self, game: &GameState) -> Result<RaidDisplayState> {
        RaidPhaseImpl::display_state(*self, game)
    }

    fn prompt_context(&self) -> Option<PromptContext> {
        RaidPhaseImpl::prompt_context(*self)
    }

    fn handle_prompt(
        &self,
        game: &mut GameState,
        action: PromptAction,
    ) -> Result<Option<InternalRaidPhase>> {
        RaidPhaseImpl::handle_prompt(*self, game, action)
    }

    fn prompts(&self, game: &GameState) -> Result<Vec<PromptAction>> {
        RaidPhaseImpl::prompts(*self, game)
    }
}
