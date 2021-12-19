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

use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use strum_macros::Display;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Display, Serialize, Deserialize)]
pub enum CardName {
    /// Empty identity cards with no associated rules text, for use in tests
    TestChampionIdentity,
    TestOverlordIdentity,

    ArcaneRecovery,
    Greataxe,
    GoldMine,
    IceDragon,
    DungeonAnnex,
}

impl CardName {
    pub fn displayed_name(&self) -> String {
        format!("{}", self).from_case(Case::Pascal).to_case(Case::Title)
    }
}

impl PartialOrd<Self> for CardName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CardName {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to_string().cmp(&other.to_string())
    }
}
