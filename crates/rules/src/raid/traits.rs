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

pub enum RaidDisplayState {
    None,
    Defenders(Vec<CardId>),
    Access,
}

pub trait RaidPhase {
    fn enter(&self, game: &mut GameState) -> Result<Option<InternalRaidPhase>>;

    fn active_side(&self) -> Side;

    fn display_state(&self, game: &GameState) -> Result<RaidDisplayState>;

    fn prompt_context(&self) -> Option<PromptContext>;

    fn handle_prompt(
        &self,
        game: &mut GameState,
        action: PromptAction,
    ) -> Result<Option<InternalRaidPhase>>;

    fn prompts(&self, game: &GameState) -> Result<Vec<PromptAction>>;
}

pub trait RaidPhaseImpl: RaidPhase + Sized + Copy {
    type Action;

    fn unwrap(action: PromptAction) -> Result<Self::Action>;

    fn wrap(action: Self::Action) -> Result<PromptAction>;

    fn enter(self, game: &mut GameState) -> Result<Option<InternalRaidPhase>>;

    fn actions(self, game: &GameState) -> Result<Vec<Self::Action>>;

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
