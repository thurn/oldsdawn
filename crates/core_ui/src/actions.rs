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

use std::fmt::Debug;

use data::game_actions::{DebugAction, PromptAction, UserAction};
use protos::spelldawn::game_action::Action;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::{CommandList, GameCommand, StandardAction};
use serde_json::ser;

/// Represents an action that can be performed in the user interface. Initiating
/// a server request and performing an immediate client update are both
/// supported forms of action.
pub trait InterfaceAction: Debug {
    fn as_game_action(&self) -> Option<Action>;
}

impl<T: InterfaceAction + Clone> InterfaceAction for Option<T> {
    fn as_game_action(&self) -> Option<Action> {
        self.as_ref().and_then(|action| action.clone().as_game_action())
    }
}

impl<T: ?Sized + InterfaceAction> InterfaceAction for Box<T> {
    fn as_game_action(&self) -> Option<Action> {
        self.as_ref().as_game_action()
    }
}

/// Marker struct for when no action is desired.
#[derive(Debug)]
pub struct NoAction {}

impl InterfaceAction for NoAction {
    fn as_game_action(&self) -> Option<Action> {
        None
    }
}

impl InterfaceAction for DebugAction {
    fn as_game_action(&self) -> Option<Action> {
        Some(Action::StandardAction(StandardAction {
            payload: payload(UserAction::Debug(*self)),
            update: None,
        }))
    }
}

impl InterfaceAction for UserAction {
    fn as_game_action(&self) -> Option<Action> {
        Some(Action::StandardAction(StandardAction { payload: payload(*self), update: None }))
    }
}

impl InterfaceAction for PromptAction {
    fn as_game_action(&self) -> Option<Action> {
        Some(Action::StandardAction(StandardAction {
            payload: payload(UserAction::GamePromptResponse(*self)),
            update: None,
        }))
    }
}

impl InterfaceAction for Command {
    fn as_game_action(&self) -> Option<Action> {
        Some(Action::StandardAction(StandardAction {
            payload: vec![],
            update: Some(command_list(vec![self.clone()])),
        }))
    }
}

impl InterfaceAction for Vec<Command> {
    fn as_game_action(&self) -> Option<Action> {
        Some(Action::StandardAction(StandardAction {
            payload: vec![],
            update: Some(command_list(self.clone())),
        }))
    }
}

fn payload(action: UserAction) -> Vec<u8> {
    ser::to_vec(&action).expect("Serialization failed")
}

fn command_list(commands: Vec<Command>) -> CommandList {
    CommandList {
        commands: commands.into_iter().map(|c| GameCommand { command: Some(c) }).collect(),
    }
}
