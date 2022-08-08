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
use clap::{ArgEnum, Parser};
use data::player_name::NamedPlayer;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
pub enum Verbosity {
    None,
    MatchOutcomes,
    Actions,
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    #[clap(arg_enum, value_parser)]
    pub overlord: NamedPlayer,
    #[clap(arg_enum, value_parser)]
    pub champion: NamedPlayer,
    #[clap(long, value_parser, default_value_t = 5)]
    pub move_time: u64,
    #[clap(long, value_parser, default_value_t = 1)]
    pub matches: u64,
    #[clap(long, value_parser, default_value = "match-outcomes")]
    pub verbosity: Verbosity,
    #[clap(long, value_parser, default_value_t = false)]
    pub deterministic: bool,
}

pub fn main() -> Result<()> {
    let _ = Args::parse();
    println!("Hello, world");
    Ok(())
}
