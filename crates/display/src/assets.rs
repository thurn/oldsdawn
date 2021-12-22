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

use data::primitives::{CardType, Faction, Rarity, School, Side};
use protos::spelldawn::SpriteAddress;

pub enum CardIconType {
    LevelCounter,
    Mana,
    Health,
    Attack,
    Shield,
}

pub fn card_icon(icon_type: CardIconType) -> SpriteAddress {
    SpriteAddress {
        address: match icon_type {
            CardIconType::LevelCounter => {
                "LittleSweetDaemon/TCG_Card_Elemental_Design/Number_Icons/Number_Icons_Color_3"
            }
            CardIconType::Mana => {
                "LittleSweetDaemon/TCG_Card_Fantasy_Design/Icons/Icon_Mana_Color_01"
            }
            CardIconType::Health => {
                "LittleSweetDaemon/TCG_Card_Elemental_Design/Heart_Icons/Heart_Icons_Color_5"
            }
            CardIconType::Attack => {
                "LittleSweetDaemon/TCG_Card_Elemental_Design/Attack_Icons/Attack_Icons_Color_4"
            }
            CardIconType::Shield => {
                "LittleSweetDaemon/TCG_Card_Elemental_Design/Number_Icons/Number_Icons_Color_6"
            }
        }
        .to_string(),
    }
}

pub fn identity_card_frame(side: Side) -> SpriteAddress {
    SpriteAddress { address: identity_card_frame_string(side) }
}

fn identity_card_frame_string(side: Side) -> String {
    match side {
        Side::Overlord => "SpriteWay/Icons/Fantasy Player Frames/50002",
        Side::Champion => "SpriteWay/Icons/Fantasy Player Frames/50003",
    }
    .to_string()
}

pub fn card_back(school: School) -> SpriteAddress {
    SpriteAddress {
        address: match school {
            School::Time => {
                "LittleSweetDaemon/TCG_Card_Fantasy_Design/Backs/Back_Steampunk_Style_Color_1"
            }
            School::Neutral => {
                "LittleSweetDaemon/TCG_Card_Fantasy_Design/Backs/Back_Lovecraft_Style_Color_1"
            }
            School::Shadow => {
                "LittleSweetDaemon/TCG_Card_Fantasy_Design/Backs/Back_Daemon_Style_Color_1"
            }
            School::Nature => {
                "LittleSweetDaemon/TCG_Card_Fantasy_Design/Backs/Back_Elf_Style_Color_1"
            }
        }
        .to_string(),
    }
}

pub fn card_frame(school: School) -> SpriteAddress {
    SpriteAddress {
        address: match school {
            School::Time => {
                "LittleSweetDaemon/TCG_Card_Fantasy_Design/Cards/Card_Steampunk_Style_Color_1"
            }
            School::Neutral => {
                "LittleSweetDaemon/TCG_Card_Fantasy_Design/Cards/Card_Lovecraft_Style_Color_1"
            }
            School::Shadow => {
                "LittleSweetDaemon/TCG_Card_Fantasy_Design/Cards/Card_Daemon_Style_Color_1"
            }
            School::Nature => {
                "LittleSweetDaemon/TCG_Card_Fantasy_Design/Cards/Card_Elf_Style_Color_1"
            }
        }
        .to_string(),
    }
}

pub fn title_background(faction: Option<Faction>) -> SpriteAddress {
    faction.map_or_else(
        || SpriteAddress {
            address: "LittleSweetDaemon/TCG_Card_Design/Warrior_Card/Warrior_Card Face_Tape"
                .to_string(),
        },
        |f| SpriteAddress {
            address: match f {
                Faction::Mortal => {
                    "LittleSweetDaemon/TCG_Card_Design/Nautical_Card/Nautical_Card_Face_Tape"
                }
                Faction::Abyssal => {
                    "LittleSweetDaemon/TCG_Card_Design/Magic_Card/Magic_Card_Face_Tape"
                }
                Faction::Infernal => {
                    "LittleSweetDaemon/TCG_Card_Design/Animal_Card/Animal_Card_Face_Tape"
                }
            }
            .to_string(),
        },
    )
}

pub fn arena_frame(side: Side, card_type: CardType, faction: Option<Faction>) -> SpriteAddress {
    faction.map_or_else(
        || SpriteAddress {
            address: match card_type {
                CardType::Spell | CardType::Minion | CardType::Upgrade => {
                    "SpriteWay/Icons/Clean Frames/9020".to_string()
                }
                CardType::Weapon | CardType::Artifact | CardType::Project | CardType::Scheme => {
                    "SpriteWay/Icons/Clean Frames/9047".to_string()
                }
                CardType::Identity => identity_card_frame_string(side),
            },
        },
        |f| SpriteAddress {
            address: match f {
                Faction::Mortal => "SpriteWay/Icons/Clean Frames/9048",
                Faction::Abyssal => "SpriteWay/Icons/Clean Frames/9055",
                Faction::Infernal => "SpriteWay/Icons/Clean Frames/9054",
            }
            .to_string(),
        },
    )
}

pub fn jewel(rarity: Rarity) -> SpriteAddress {
    SpriteAddress {
        address: match rarity {
            Rarity::Common | Rarity::None => {
                "LittleSweetDaemon/TCG_Card_Fantasy_Design/Jewels/Jewel_Elf_Color_01"
            }
            Rarity::Uncommon => {
                "LittleSweetDaemon/TCG_Card_Fantasy_Design/Jewels/Jewel_Steampunk_Color_01"
            }
            Rarity::Rare => "LittleSweetDaemon/TCG_Card_Fantasy_Design/Jewels/Jewel_Elf_Color_02",
            Rarity::Epic => {
                "LittleSweetDaemon/TCG_Card_Fantasy_Design/Jewels/Jewel_Steampunk_Color_02"
            }
        }
        .to_string(),
    }
}
