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

//! Helper functions for constructing resource URLs used during a game

use core_ui::design::FontColor;
use data::primitives::{CardType, Faction, Rarity, School, Side};
use data::special_effects::{
    FantasyEventSounds, FireworksSound, Projectile, SoundEffect, TimedEffect,
};
use protos::spelldawn::{
    AudioClipAddress, EffectAddress, FlexColor, ProjectileAddress, SpriteAddress,
};

/// Possible types of icons which can appear on a card
pub enum CardIconType {
    LevelCounter,
    Mana,
    Health,
    Attack,
    Shield,
    LevelRequirement,
    Points,
}

/// Returns the background scale multiplier to use for a [CardIconType]
pub fn background_scale(icon_type: CardIconType) -> Option<f32> {
    Some(match icon_type {
        CardIconType::Health => 1.5,
        CardIconType::Attack => 1.75,
        CardIconType::Shield => 1.1,
        CardIconType::LevelRequirement => 0.9,
        CardIconType::Points => 0.35,
        _ => 1.0,
    })
}

/// Address for a given [CardIconType]
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
            CardIconType::LevelRequirement => {
                "LittleSweetDaemon/TCG_Card_Elemental_Design/Number_Back/Number_Back_Color_3"
            }
            CardIconType::Points => {
                "LittleSweetDaemon/TCG_Card_Elemental_Design/Card_Color_07/Back_Card_Color_07/Back_Card_Color_07_Logo_Crystal"
            }
        }
        .to_string(),
    }
}

/// Address for the frame of a player's identity card image
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

/// Address for the back of a card of a given [School]
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

/// Address for the frame of a card of a given [School]
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

pub fn ability_card_frame(side: Side) -> SpriteAddress {
    SpriteAddress {
        address: match side {
            Side::Overlord => "LittleSweetDaemon/TCG_Card_Design/Custom/OverlordFront",
            Side::Champion => "LittleSweetDaemon/TCG_Card_Design/Custom/ChampionFront",
        }
        .to_string(),
    }
}

/// Title font color to use for a given [Faction].
pub fn title_color(faction: Option<Faction>) -> FlexColor {
    match faction {
        None => FontColor::NormalCardTitle,
        Some(Faction::Mortal) => FontColor::MortalCardTitle,
        Some(Faction::Infernal) => FontColor::InfernalCardTitle,
        Some(Faction::Abyssal) => FontColor::AbyssalCardTitle,
        Some(Faction::Prismatic) => FontColor::PrismaticCardTitle,
        Some(Faction::Construct) => FontColor::ConstructCardTitle,
    }
    .into()
}

/// Address for an image to display as a background for a card of the given
/// [Faction].
pub fn title_background(_: Option<Faction>) -> SpriteAddress {
    SpriteAddress {
        address: "LittleSweetDaemon/TCG_Card_Design/Custom/Title/BlackWhiteFaceTape".to_string(),
    }
}

/// Address for the frame of a card in the arena
pub fn arena_frame(side: Side, card_type: CardType, faction: Option<Faction>) -> SpriteAddress {
    faction.map_or_else(
        || SpriteAddress {
            address: match card_type {
                CardType::OverlordSpell | CardType::ChampionSpell | CardType::Minion => {
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
                Faction::Prismatic => "SpriteWay/Icons/Clean Frames/9048",
                Faction::Mortal => "SpriteWay/Icons/Clean Frames/9048",
                Faction::Abyssal => "SpriteWay/Icons/Clean Frames/9055",
                Faction::Infernal => "SpriteWay/Icons/Clean Frames/9054",
                Faction::Construct => "SpriteWay/Icons/Clean Frames/9020",
            }
            .to_string(),
        },
    )
}

/// Address for the rarity jewel to display on a card
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

pub fn projectile(projectile: Projectile) -> ProjectileAddress {
    ProjectileAddress {
        address: match projectile {
            Projectile::Hovl(number) => format!(
                "Hovl Studio/AAA Projectiles Vol 1/Prefabs/Projectiles/Projectile {}",
                number
            ),
        },
    }
}

pub fn timed_effect(effect: TimedEffect) -> EffectAddress {
    EffectAddress {
        address: match effect {
            TimedEffect::HovlMagicHit(number) => {
                format!("Hovl Studio/Magic hits/Prefabs/Hit {}", number)
            }
            TimedEffect::HovlSwordSlash(number) => {
                format!("Hovl Studio/Sword slash VFX/Prefabs/Sword Slash {}", number)
            }
        },
    }
}

pub fn sound_effect(effect: SoundEffect) -> AudioClipAddress {
    AudioClipAddress {
        address: match effect {
            SoundEffect::FantasyEvents(events) => match events {
                FantasyEventSounds::Positive1 => {
                    "Cafofo/Fantasy Music Pack Vol 1/Events/Positive Event 01".to_string()
                }
            },
            SoundEffect::Fireworks(firework) => match firework {
                FireworksSound::RocketExplodeLarge => {
                    "Universal Sound FX/FIREWORKS/FIREWORKS_Rocket_Explode_Large_RR1_mono"
                        .to_string()
                }
                FireworksSound::RocketExplode => {
                    "Universal Sound FX/FIREWORKS/FIREWORKS_Rocket_Explode_RR1_mono".to_string()
                }
            },
        },
    }
}
