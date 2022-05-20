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

//! Defines card names

use std::cmp::Ordering;

use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};
use strum_macros::Display;

/// Possible names of cards.
///
/// This enum is used to connect the state of a card to its game rules.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone, Display, Serialize, Deserialize)]
pub enum CardName {
    // Cards for use in tests
    TestChampionIdentity,
    TestOverlordIdentity,
    TestChampionSpell,
    TestOverlordSpell,
    /// Scheme requiring 3 levels to score 1 point
    TestScheme31,
    /// Minion with 5 health, 3 mana cost, and an "end the raid" ability.
    TestMinionEndRaid,
    /// Equivalent to `TestMinionEndRaid` with 1 shield point.
    TestMinionShield1Infernal,
    /// Equivalent to `TestMinionEndRaid` with 2 shield point & abyssal faction
    TestMinionShield2Abyssal,
    /// Minion with 5 health, 1 mana cost, and a "deal 1 damage" ability.
    TestMinionDealDamage,
    /// Minion with the 'infernal' faction, MINION_HEALTH health, and an 'end
    /// raid' ability.
    TestInfernalMinion,
    /// Minion with the 'abyssal' faction, MINION_HEALTH health, and an 'end
    /// raid' ability.
    TestAbyssalMinion,
    /// Minion with the 'mortal' faction, MINION_HEALTH health, and an 'end
    /// raid' ability.
    TestMortalMinion,
    /// Weapon with 2 attack and no boost.
    TestWeapon2Attack,
    /// Weapon with 2 attack and a '1 mana: +2 attack' boost.
    TestWeapon2Attack12Boost,
    /// Weapon with 3 attack and a '1 mana: +2 attack' boost.
    TestWeapon3Attack12Boost3Cost,
    /// Weapon with 4 attack and a '1 mana: +2 attack' boost.
    TestWeapon4Attack12Boost,
    /// Weapon with 5 attack and no boost
    TestWeapon5Attack,
    /// Artifact which stores mana on play, with the activated ability to take
    /// mana from it
    TestActivatedAbilityTakeMana,
    /// Project which stores mana on unveil, with a triggered ability to take
    /// mana at dusk.
    TestTriggeredAbilityTakeManaAtDusk,

    ArcaneRecovery,
    Greataxe,
    Lodestone,
    GoldMine,
    IceDragon,
    DungeonAnnex,
    Meditation,
    CoupDeGrace,
    ChargedStrike,
    StealthMission,
    Preparation,
    SanctumPassage,
    Accumulator,
    MysticPortal,
    StorageCrystal,
    MagicalResonator,
    DarkGrimoire,
    MaraudersAxe,
    KeenHalberd,
    EtherealBlade,
    BowOfTheAlliance,
    ActivateReinforcements,
    ResearchProject,
    Gemcarver,
    Coinery,
    PitTrap,
}

impl CardName {
    /// Returns the user-visible name for this card
    pub fn displayed_name(&self) -> String {
        match self {
            Self::MaraudersAxe => "Marauder's Axe".to_string(),
            _ => format!("{}", self).from_case(Case::Pascal).to_case(Case::Title),
        }
    }

    /// Returns true if this card is a test blank
    pub fn is_test_card(&self) -> bool {
        self.displayed_name().starts_with("Test")
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
