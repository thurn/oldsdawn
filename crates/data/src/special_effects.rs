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

//! Types which describe custom visual & sound effects used during play

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Projectile {
    /// Hovl Studios projectile number
    Hovl(u32),
}

/// Effect which plays for a short duration and then vanishes
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum TimedEffect {
    /// Magic hit number
    HovlMagicHit(u32),
    /// Sword Slash VFX number
    HovlSwordSlash(u32),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum FireworksSound {
    RocketExplodeLarge,
    RocketExplode,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum FantasyEventSounds {
    Positive1,
}

/// Plays a sound
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum SoundEffect {
    FantasyEvents(FantasyEventSounds),
    Fireworks(FireworksSound),
}
