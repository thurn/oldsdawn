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
    Sprite {
        address: format!("{}.png", vec!["Rexard".to_string(), pack.path(), name.into()].join("/")),
    }
}

pub fn weapon(weapon_type: RexardWeaponType, name: impl Into<String>) -> Sprite {
    Sprite {
        address: format!(
            "{}.png",
            vec![
                "Rexard/FantasyIconsMegaPack/WeaponsIcons/WeaponsIcons_png/black".to_string(),
                weapon_type.path(),
                name.into(),
            ]
            .join("/")
        ),
    }
}

pub fn artifact(armor_type: RexardArtifactType, name: impl Into<String>) -> Sprite {
    Sprite {
        address: format!(
            "{}.png",
            vec![
                "Rexard/FantasyIconsMegaPack/ArmorIcons/ArmorIcons_png/black".to_string(),
                armor_type.path(),
                name.into(),
            ]
            .join("/")
        ),
    }
}

pub fn spell(page: u32, name: impl Into<String>) -> Sprite {
    Sprite {
        address: format!("Rexard/SpellBookMegapack/SpellBookPage0{}/{}.png", page, name.into()),
    }
}

pub enum RexardPack {
    MonstersAvatars,
    MagicItems,
    JeweleryRings,
    JeweleryNecklaces,
    MiningIcons,
    LootIcons,
}

impl RexardPack {
    fn path(&self) -> String {
        match self {
            Self::MonstersAvatars => "MonstersAvatarIcons/png",
            Self::MagicItems => "FantasyIconsMegaPack/MagicItems/MagicItems_png/bg",
            Self::JeweleryRings => "FairytaleIconsMegapack/JewelryIcons/bg/rings",
            Self::JeweleryNecklaces => "FairytaleIconsMegapack/JewelryIcons/bg/necklaces",
            Self::MiningIcons => "FairytaleIconsMegapack/MiningIcons/MiningIcons_bg",
            Self::LootIcons => "FantasyIconsMegaPack/LootIcons/LootIcons_png/black",
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

pub enum RexardArtifactType {
    Addons,
    Armor,
    Belts,
    Boots,
    Bracers,
    Cloaks,
    Gloves,
    Helmets,
    Necklace,
    Pants,
    Rings,
    Shoulders,
}

impl RexardArtifactType {
    fn path(&self) -> String {
        match self {
            Self::Addons => "addons",
            Self::Armor => "armor",
            Self::Belts => "belts",
            Self::Boots => "boots",
            Self::Bracers => "bracers",
            Self::Cloaks => "cloaks",
            Self::Gloves => "gloves",
            Self::Helmets => "helmets",
            Self::Necklace => "necklace",
            Self::Pants => "pants",
            Self::Rings => "rings",
            Self::Shoulders => "shoulders",
        }
        .to_string()
    }
}
