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

//! Helpers to construct URLs for card images

use data::primitives::Sprite;

/// Builds a sprite URL for a given card image
pub fn get(pack: RexardPack, name: impl Into<String>) -> Sprite {
    Sprite { address: vec!["Rexard".to_string(), pack.path(), name.into()].join("/") }
}

/// Builds a sprite URL for a given card image
pub fn get_weapon(weapon_type: RexardWeaponType, name: impl Into<String>) -> Sprite {
    Sprite {
        address: vec![
            "Rexard/FantasyIconsMegaPack/WeaponsIcons/WeaponsIcons_png/black".to_string(),
            weapon_type.path(),
            name.into(),
        ]
        .join("/"),
    }
}

pub enum RexardPack {
    MonstersAvatars,
}

impl RexardPack {
    fn path(&self) -> String {
        match self {
            Self::MonstersAvatars => "MonstersAvatarIcons/png",
        }
        .to_string()
    }
}

pub enum RexardWeaponType {
    Ammunition,
    Axes,
    Bows,
    BrassKnuckles,
    Clubs,
    Crossbows,
    Daggers,
    Guns,
    Hammers,
    Polearms,
    Staves,
    Swords,
    ThrowingWeapons,
}

impl RexardWeaponType {
    fn path(&self) -> String {
        match self {
            Self::Ammunition => "ammunition",
            Self::Axes => "axes",
            Self::Bows => "bows",
            Self::BrassKnuckles => "brass_knuckles",
            Self::Clubs => "clubs",
            Self::Crossbows => "crossbows",
            Self::Daggers => "daggers",
            Self::Guns => "guns",
            Self::Hammers => "hammers",
            Self::Polearms => "polearms",
            Self::Staves => "staves",
            Self::Swords => "swords",
            Self::ThrowingWeapons => "throwing_weapons",
        }
        .to_string()
    }
}
