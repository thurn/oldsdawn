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

use ai::tournament::run_tournament;
use ai::tournament::run_tournament::RunGames;
use cards::{decklists, initialize};
use data::agent_definition::AgentName;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Running AI match");
    initialize::run();
    let mut game = decklists::canonical_game()?;
    run_tournament::run_games(
        &mut game,
        1,
        AgentName::PickRandom,
        AgentName::PickRandom,
        RunGames::PrintActions,
    );

    Ok(())
}
