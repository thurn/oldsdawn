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

use std::io;

use ai_core::agent::Agent;
use ai_core::game_state_node::GameStateNode;
use ai_testing::nim::{nim_sum, NimAction, NimPile, NimPlayer, NimState};
use ai_testing::nim_agents::{NIM_MINIMAX_AGENT, NIM_PERFECT_AGENT};
use anyhow::Result;
use clap::{ArgEnum, Parser};
use with_error::WithError;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(arg_enum, value_parser)]
    pub player_one: NimAgentName,
    #[clap(arg_enum, value_parser)]
    pub player_two: NimAgentName,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum NimAgentName {
    Human,
    Perfect,
    Minimax,
}

pub fn main() -> Result<()> {
    let args = Args::parse();
    println!("Welcome to the Game of Nim");
    let nim = NimState::new(4);
    run_game_loop(nim, get_agent(args.player_one), get_agent(args.player_two))
}

fn get_agent(name: NimAgentName) -> Box<dyn Agent<NimState>> {
    match name {
        NimAgentName::Human => Box::new(NimHumanAgent {}),
        NimAgentName::Perfect => Box::new(NIM_PERFECT_AGENT),
        NimAgentName::Minimax => Box::new(NIM_MINIMAX_AGENT),
    }
}

fn run_game_loop(
    mut nim: NimState,
    player_one: Box<dyn Agent<NimState>>,
    player_two: Box<dyn Agent<NimState>>,
) -> Result<()> {
    loop {
        print_optimal_action(&nim, player_one.name())?;
        println!("{}", nim);
        let p1_action = player_one.pick_action(&nim)?;
        println!("<<{}>> takes {} from {}", player_one.name(), p1_action.amount, p1_action.pile);
        nim.execute_action(NimPlayer::One, p1_action)?;
        check_game_over(&nim, player_one.name(), player_two.name());

        print_optimal_action(&nim, player_two.name())?;
        println!("{}", nim);

        let p2_action = player_two.pick_action(&nim)?;
        println!("<<{}>> takes {} from {}", player_two.name(), p2_action.amount, p2_action.pile);
        nim.execute_action(NimPlayer::Two, p2_action)?;
        check_game_over(&nim, player_one.name(), player_two.name());
    }
}

fn print_optimal_action(state: &NimState, player_name: &str) -> Result<()> {
    if nim_sum(state) == 0 {
        println!("  (Game is unwinnable for {} with optimal play)", player_name);
    } else {
        let action = NIM_PERFECT_AGENT.pick_action(state)?;
        println!("  (Optimal play for {} is {} take {})", player_name, action.pile, action.amount);
    }

    Ok(())
}

fn check_game_over(state: &NimState, p1_name: &str, p2_name: &str) {
    if state.current_turn().is_none() {
        match state.turn {
            NimPlayer::One => println!("Game Over. {} wins!", p2_name),
            NimPlayer::Two => println!("Game Over. {} wins!", p1_name),
        }
        std::process::exit(0)
    }
}

struct NimHumanAgent;

impl Agent<NimState> for NimHumanAgent {
    fn name(&self) -> &'static str {
        "HUMAN"
    }

    fn pick_action(&self, state: &NimState) -> Result<NimAction> {
        println!("\n>>> Input your action, e.g. 'a2' or 'b3'");

        let mut input_text = String::new();
        io::stdin().read_line(&mut input_text)?;

        let trimmed = input_text.trim();
        assert_eq!(trimmed.len(), 2);
        let characters = trimmed.chars().collect::<Vec<_>>();
        let pile = match characters[0] {
            'a' => NimPile::PileA,
            'b' => NimPile::PileB,
            'c' => NimPile::PileC,
            _ => panic!("Input must be a, b, or c"),
        };
        let amount = characters[1].to_digit(10).with_error(|| "Input must be 1-9")?;
        assert!(amount > 0);
        assert!(amount <= state.piles[&pile]);

        Ok(NimAction { pile, amount })
    }
}
