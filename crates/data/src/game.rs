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

//! Core data structures for tracking the state of an ongoing game.

#![allow(clippy::use_self)] // Required to use EnumKind

use std::collections::HashMap;

use anyhow::Result;
use rand_xoshiro::rand_core::SeedableRng;
use rand_xoshiro::Xoshiro256StarStar;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use with_error::WithError;

use crate::card_state::{AbilityState, CardPosition, CardPositionKind, CardState};
use crate::deck::Deck;
use crate::delegates::DelegateCache;
use crate::game_actions::GamePrompt;
use crate::player_name::PlayerId;
use crate::primitives::{
    AbilityId, ActionCount, CardId, GameId, HasAbilityId, ItemLocation, ManaValue, PointsValue,
    RaidId, RoomId, RoomLocation, Side, TurnNumber,
};
use crate::updates::{GameUpdate, UpdateStep, UpdateTracker, Updates};

/// Mana to be spent only during the `raid_id` raid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificRaidMana {
    pub raid_id: RaidId,
    pub mana: ManaValue,
}

/// Stores a player's mana, both a general-purpose pool and various
/// restricted-purpose pools.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ManaState {
    /// General mana, can be used for any purpose.
    pub base_mana: ManaValue,

    /// Mana which can be used only during a specific raid.
    pub specific_raid_mana: Option<SpecificRaidMana>,
}

/// State of a player within a game, containing their score and available
/// resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: PlayerId,
    pub mana_state: ManaState,
    pub actions: ActionCount,
    pub score: PointsValue,

    /// A choice this player is facing in resolving a card ability. Takes
    /// precedence over other choices such as raid actions.
    pub prompt: Option<GamePrompt>,
}

impl PlayerState {
    /// Create an empty player state.
    pub fn new(id: PlayerId) -> Self {
        Self { id, mana_state: ManaState::default(), actions: 0, score: 0, prompt: None }
    }
}

/// Current internal state of the raid state machine. Use the methods of the
/// `RaidPhase` trait to interact with an ongoing raid.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq)]
pub enum InternalRaidPhase {
    /// Raid has been created
    Begin,
    /// The defender with the provided index position is currently being
    /// encountered. The Champion is deciding which weapons, if any, to employ.
    ///
    /// Note that defenders are encountered in decreasing position order.
    Encounter,
    /// The Champion has bypassed all of the defenders for this room and is now
    /// accessing its contents
    Access,
}

/// Some card abilities completely change the state of a Raid, for example to
/// target a different room or encounter a different minion. Setting a jump
/// request on the [RaidData] asks the raid system to execute the related
/// transformation when possible.
///
/// This always happens *after* processing the current raid state and only if
/// the raid is still active, i.e. it cannot be used to stop the effects of a
/// user action from happening.
///
/// Only one jump request is supported at a time, on a 'last write wins' basis.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum RaidJumpRequest {
    EncounterMinion(CardId),
}

/// Data about an active raid
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaidData {
    /// Unique ID for this raid
    pub raid_id: RaidId,
    /// Room being targeted by this raid
    pub target: RoomId,
    /// Current state of this raid. Please use the `.phase()` method of the
    /// `RaidDataExt` trait instead of examining this directly.
    pub internal_phase: InternalRaidPhase,
    /// Current encounter position within this raid, if any
    pub encounter: Option<usize>,
    /// Cards which have been accessed as part of this raid's Access phase.
    pub accessed: Vec<CardId>,
    /// Requested new state for this raid. See [RaidJumpRequest] for details.
    pub jump_request: Option<RaidJumpRequest>,
}

/// Describes options for this game & the set of rules it is using.
#[derive(Debug, Clone, Default, Copy, Serialize, Deserialize)]
pub struct GameConfiguration {
    /// If true, all random choices within this game will be made
    /// deterministically using a seeded random number generator. Useful for
    /// e.g. unit tests.
    pub deterministic: bool,
    /// Whether to run in simulation mode and thus disable update tracking
    pub simulation: bool,
}

/// Mulligan decision a player made for their opening hand
#[derive(Debug, Copy, Clone, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum MulliganDecision {
    /// The player has decided to keep their initial hand of 5 cards
    Keep,
    /// The player has elected to draw a new hand of 5 cards
    Mulligan,
}

/// [MulliganDecision]s for both players.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MulliganData {
    /// The mulligan decision for the Overlord player, or None if no decision
    /// has been made.
    pub overlord: Option<MulliganDecision>,
    /// The mulligan decision for the Champion player, or None if no decision
    /// has been made.
    pub champion: Option<MulliganDecision>,
}

impl MulliganData {
    pub fn decision(&self, side: Side) -> Option<&MulliganDecision> {
        match side {
            Side::Overlord => &self.overlord,
            Side::Champion => &self.champion,
        }
        .as_ref()
    }
}

/// Identifies the player whose turn it is
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct TurnData {
    /// Current player whose turn it is
    pub side: Side,
    /// Turn number for that player
    pub turn_number: TurnNumber,
}

/// High level status of a game, including e.g. whose turn it is
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GamePhase {
    ResolveMulligans(MulliganData),
    Play,
    GameOver { winner: Side },
}

/// State and configuration of the overall game, including whose turn it is and
/// whether a raid is active.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameData {
    /// Current [GamePhase].
    pub phase: GamePhase,
    /// Identifies current game turn
    pub turn: TurnData,
    /// Data about an ongoing raid, if any
    pub raid: Option<RaidData>,
    /// Counter to create unique IDs for raids within this game
    pub next_raid_id: u32,
    /// Game options
    pub config: GameConfiguration,
}

/// State for an individual room
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RoomState {
    /// When was a raid last initiated for this room?
    pub last_raided: Option<TurnData>,
}

/// Stores the primary state for an ongoing game
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Unique identifier for this game
    pub id: GameId,
    /// General game state & configuration
    pub data: GameData,
    /// Used to track changes to game state in order to update the client. See
    /// [UpdateTracker] for more information.
    #[serde(skip)]
    pub updates: UpdateTracker,
    /// Cards for the overlord player. In general, code should use one of the
    /// helper methods below instead of accessing this directly.
    pub overlord_cards: Vec<CardState>,
    /// Cards for the champion player. In general, code should use one of the
    /// helper methods below instead of accessing this directly.
    pub champion_cards: Vec<CardState>,
    /// State for the overlord player
    pub overlord: PlayerState,
    /// State for the champion player
    pub champion: PlayerState,
    /// State for abilities of cards in this game
    #[serde_as(as = "Vec<(_, _)>")]
    pub ability_state: HashMap<AbilityId, AbilityState>,
    /// State for rooms
    #[serde_as(as = "Vec<(_, _)>")]
    #[serde(default)]
    pub room_state: HashMap<RoomId, RoomState>,
    /// Next sorting key to use for card moves. Automatically updated by
    /// [Self::next_sorting_key] and [Self::move_card_internal].
    next_sorting_key: u32,
    /// Optionally, a random number generator for this game to use. This
    /// generator is serializable, so the state will be deterministic even
    /// across different sessions. If not specified, `rand::thread_rng()` is
    /// used.
    pub rng: Option<Xoshiro256StarStar>,
    /// Optional lookup table for delegates present on cards in this game in
    /// order to improve performance
    #[serde(skip)]
    pub delegate_cache: DelegateCache,
}

impl GameState {
    /// Creates a new game with the provided [GameId] and decks for both players
    /// in the [GamePhase::ResolveMulligans] phase.
    ///
    /// Does *not* handle dealing opening hands, prompting for mulligan
    /// decisions, assigning starting mana, etc.
    pub fn new(
        id: GameId,
        overlord_deck: Deck,
        champion_deck: Deck,
        config: GameConfiguration,
    ) -> Self {
        Self {
            id,
            data: GameData {
                phase: GamePhase::ResolveMulligans(MulliganData::default()),
                turn: TurnData { side: Side::Overlord, turn_number: 0 },
                raid: None,
                next_raid_id: 1,
                config,
            },
            overlord_cards: Self::make_deck(&overlord_deck, Side::Overlord),
            champion_cards: Self::make_deck(&champion_deck, Side::Champion),
            overlord: PlayerState::new(overlord_deck.owner_id),
            champion: PlayerState::new(champion_deck.owner_id),
            ability_state: HashMap::new(),
            room_state: HashMap::new(),
            updates: UpdateTracker::new(if config.simulation {
                Updates::Ignore
            } else {
                Updates::Push
            }),
            next_sorting_key: 1,
            delegate_cache: DelegateCache::default(),
            rng: if config.deterministic {
                Some(Xoshiro256StarStar::seed_from_u64(314159265358979323))
            } else {
                None
            },
        }
    }

    pub fn record_update(&mut self, update: impl FnOnce() -> GameUpdate) {
        if self.updates.state == Updates::Push {
            // Snapshot current game state, omit things that aren't important for display
            // logic.
            let clone = Self {
                id: self.id,
                data: self.data.clone(),
                updates: UpdateTracker::new(Updates::Ignore),
                overlord_cards: self.overlord_cards.clone(),
                champion_cards: self.champion_cards.clone(),
                overlord: self.overlord.clone(),
                champion: self.champion.clone(),
                ability_state: self.ability_state.clone(),
                room_state: self.room_state.clone(),
                next_sorting_key: self.next_sorting_key,
                rng: None,
                delegate_cache: DelegateCache::default(),
            };

            self.updates.steps.push(UpdateStep { snapshot: clone, update: update() });
        }
    }

    /// Makes a clone of the game state without including the [UpdateTracker]
    /// data.
    pub fn clone_without_updates(&self) -> Self {
        Self {
            id: self.id,
            data: self.data.clone(),
            updates: UpdateTracker::default(),
            overlord_cards: self.overlord_cards.clone(),
            champion_cards: self.champion_cards.clone(),
            overlord: self.overlord.clone(),
            champion: self.champion.clone(),
            ability_state: self.ability_state.clone(),
            room_state: self.room_state.clone(),
            next_sorting_key: self.next_sorting_key,
            rng: self.rng.clone(),
            delegate_cache: self.delegate_cache.clone(),
        }
    }

    /// Returns identity cards for the provided Side
    pub fn identities(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position().kind() == CardPositionKind::Identity)
    }

    /// Returns the first identity for the `side` player.
    pub fn first_identity(&self, side: Side) -> Result<CardId> {
        let identities = self.card_list_for_position(side, CardPosition::Identity(side));
        Ok(*identities.first().with_error(|| "No identity found")?)
    }

    /// Returns an arbitrary identity card for the provided `side`, if any.
    pub fn some_identity(&self, side: Side) -> Result<&CardState> {
        self.identities(side).next().with_error(|| format!("No identity card for {:?}", side))
    }

    /// Look up [CardState] for a card. Panics if this card is not present in
    /// the game.
    pub fn card(&self, card_id: CardId) -> &CardState {
        &self.cards(card_id.side)[card_id.index]
    }

    /// Mutable version of [Self::card]
    pub fn card_mut(&mut self, card_id: CardId) -> &mut CardState {
        &mut self.cards_mut(card_id.side)[card_id.index]
    }

    /// Cards for a player, in an unspecified order
    pub fn cards(&self, side: Side) -> &Vec<CardState> {
        match side {
            Side::Overlord => &self.overlord_cards,
            Side::Champion => &self.champion_cards,
        }
    }

    /// Mutable version of [Self::cards]
    pub fn cards_mut(&mut self, side: Side) -> &mut Vec<CardState> {
        match side {
            Side::Overlord => &mut self.overlord_cards,
            Side::Champion => &mut self.champion_cards,
        }
    }

    /// State for the players in the game
    pub fn player(&self, side: Side) -> &PlayerState {
        match side {
            Side::Overlord => &self.overlord,
            Side::Champion => &self.champion,
        }
    }

    /// Mutable version of [Self::player]
    pub fn player_mut(&mut self, side: Side) -> &mut PlayerState {
        match side {
            Side::Overlord => &mut self.overlord,
            Side::Champion => &mut self.champion,
        }
    }

    /// Returns a monotonically-increasing sorting key for object positions in
    /// this game.
    pub fn next_sorting_key(&mut self) -> u32 {
        let result = self.next_sorting_key;
        self.next_sorting_key += 1;
        result
    }

    /// Moves a card to a new [CardPosition], updating its sorting key.
    ///
    /// Generally use `mutations::move_card` instead of calling this method
    /// directly.
    pub fn move_card_internal(&mut self, card_id: CardId, new_position: CardPosition) {
        let key = self.next_sorting_key();
        self.card_mut(card_id).set_position_internal(key, new_position);
    }

    /// Moves a card to a given `index` location within its [CardPosition],
    /// shifting all elements after it to the right.
    ///
    /// Moves the card to the end of the list if `index` is out of bounds.
    pub fn move_card_to_index(&mut self, card_id: CardId, mut index: usize) {
        let mut cards = self.card_list_for_position(card_id.side, self.card(card_id).position());
        if index > cards.len() - 1 {
            index = cards.len() - 1;
        }

        cards.retain(|id| *id != card_id);
        cards.insert(index, card_id);

        for id in cards {
            self.card_mut(id).sorting_key = self.next_sorting_key();
        }
    }

    /// Cards owned by a given player in a given position, in an unspecified
    /// order
    pub fn cards_in_position(
        &self,
        side: Side,
        position: CardPosition,
    ) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(move |c| c.position() == position)
    }

    pub fn cards_in_position_mut(
        &mut self,
        side: Side,
        position: CardPosition,
    ) -> impl Iterator<Item = &mut CardState> {
        self.cards_mut(side).iter_mut().filter(move |c| c.position() == position)
    }

    /// Cards owned by a player in a given position, in sorting-key order
    pub fn card_list_for_position(&self, side: Side, position: CardPosition) -> Vec<CardId> {
        let mut result = self.cards_in_position(side, position).collect::<Vec<_>>();
        result.sort();
        result.iter().map(|c| c.id).collect()
    }

    /// Cards in a player's hand, in an unspecified order
    pub fn hand(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position().in_hand())
    }

    /// Cards in a player's deck, in an unspecified order
    pub fn deck(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position().in_deck())
    }

    /// Cards in a player's discard pile, in an unspecified order
    pub fn discard_pile(&self, side: Side) -> impl Iterator<Item = &CardState> {
        self.cards(side).iter().filter(|c| c.position().in_discard_pile())
    }

    /// Returns Overlord cards defending a given room in an unspecified order
    pub fn defenders_unordered(&self, room_id: RoomId) -> impl Iterator<Item = &CardState> {
        self.cards_in_position(Side::Overlord, CardPosition::Room(room_id, RoomLocation::Defender))
    }

    /// Overlord cards defending a given room, in sorting-key order (higher
    /// array indices are closer to the front of the room).
    pub fn defender_list(&self, room_id: RoomId) -> Vec<CardId> {
        self.card_list_for_position(
            Side::Overlord,
            CardPosition::Room(room_id, RoomLocation::Defender),
        )
    }

    /// Overlord cards in a given room (not defenders), in an unspecified order
    pub fn occupants(&self, room_id: RoomId) -> impl Iterator<Item = &CardState> {
        self.cards_in_position(Side::Overlord, CardPosition::Room(room_id, RoomLocation::Occupant))
    }

    /// All Overlord cards located within a given room, defenders and occupants,
    /// in an unspecified order.
    pub fn defenders_and_occupants(&self, room_id: RoomId) -> impl Iterator<Item = &CardState> {
        self.cards(Side::Overlord)
            .iter()
            .filter(move |c| matches!(c.position(), CardPosition::Room(r, _) if r == room_id))
    }

    /// All overlord defenders in play, whether face-up or face-down.
    pub fn minions(&self) -> impl Iterator<Item = &CardState> {
        self.cards(Side::Overlord)
            .iter()
            .filter(move |c| matches!(c.position(), CardPosition::Room(_, RoomLocation::Defender)))
    }

    /// Champion cards which have been played as weapons, in an unspecified
    /// order
    pub fn weapons(&self) -> impl Iterator<Item = &CardState> {
        self.cards_in_position(Side::Champion, CardPosition::ArenaItem(ItemLocation::Weapons))
    }

    /// Champion cards which have been played as artifacts, in an unspecified
    /// order
    pub fn artifacts(&self) -> impl Iterator<Item = &CardState> {
        self.cards_in_position(Side::Champion, CardPosition::ArenaItem(ItemLocation::Artifacts))
    }

    /// All Card IDs present in this game.
    ///
    /// Overlord cards in an unspecified order followed by Champion cards in
    /// an unspecified order.
    pub fn all_card_ids(&self) -> impl Iterator<Item = CardId> {
        (0..self.overlord_cards.len())
            .map(|index| CardId::new(Side::Overlord, index))
            .chain((0..self.champion_cards.len()).map(|index| CardId::new(Side::Champion, index)))
    }

    /// All cards in this game.
    ///
    /// Overlord cards in an unspecified order followed by Champion cards in
    /// an unspecified order.
    pub fn all_cards(&self) -> impl Iterator<Item = &CardState> {
        self.overlord_cards.iter().chain(self.champion_cards.iter())
    }

    /// Helper method to return the current [RaidData] or an error when one is
    /// expected to exist.
    pub fn raid(&self) -> Result<&RaidData> {
        self.data.raid.as_ref().with_error(|| "Expected Raid")
    }

    /// Mutable version of [Self::raid].
    pub fn raid_mut(&mut self) -> Result<&mut RaidData> {
        self.data.raid.as_mut().with_error(|| "Expected Raid")
    }

    /// Helper method to return the current raid encounter position or an error
    /// when one is expected to exist.
    pub fn raid_encounter(&self) -> Result<usize> {
        self.raid()?.encounter.with_error(|| "Expected Active Encounter")
    }

    /// Helper method to return the defender currently being encountered during
    /// a raid. Returns an error if there is no active raid or no defender is
    /// being encountered.
    pub fn raid_defender(&self) -> Result<CardId> {
        Ok(*self
            .defender_list(self.raid()?.target)
            .get(self.raid_encounter()?)
            .with_error(|| "Defender Not Found")?)
    }

    /// Retrieves the [AbilityState] for an [AbilityId]
    pub fn ability_state(&self, ability_id: impl HasAbilityId) -> Option<&AbilityState> {
        self.ability_state.get(&ability_id.ability_id())
    }

    /// Returns a mutable [AbilityState] for an [AbilityId], creating a new one
    /// if one has not previously been set
    pub fn ability_state_mut(&mut self, ability_id: impl HasAbilityId) -> &mut AbilityState {
        self.ability_state.entry(ability_id.ability_id()).or_insert_with(AbilityState::default)
    }

    /// Create card states for a deck
    fn make_deck(deck: &Deck, side: Side) -> Vec<CardState> {
        let mut result =
            vec![CardState::new(CardId::new(side, 0), deck.identity, true /* is_identity */)];

        result.extend(deck.card_names().iter().enumerate().map(move |(index, name)| {
            CardState::new(CardId::new(side, index + 1), *name, false /* is_identity */)
        }));

        result
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;
    use crate::card_name::CardName;
    use crate::player_name::NamedPlayer;

    #[test]
    fn insert_at_index() {
        let (abyssal, infernal, mortal) = (
            CardId::new(Side::Overlord, 0),
            CardId::new(Side::Overlord, 1),
            CardId::new(Side::Overlord, 2),
        );
        let mut g = test_game(
            vec![
                CardName::TestAbyssalMinion,
                CardName::TestInfernalMinion,
                CardName::TestMortalMinion,
            ],
            vec![],
        );

        fn hand(g: &GameState) -> Vec<CardId> {
            g.card_list_for_position(Side::Overlord, CardPosition::Hand(Side::Overlord))
        }

        fn hand_key_count(g: &GameState) -> usize {
            hand(g).iter().map(|id| g.card(*id).sorting_key).collect::<HashSet<_>>().len()
        }

        g.move_card_internal(abyssal, CardPosition::Hand(Side::Overlord));
        g.move_card_internal(infernal, CardPosition::Hand(Side::Overlord));
        g.move_card_internal(mortal, CardPosition::Hand(Side::Overlord));
        assert_eq!(3, hand_key_count(&g));
        assert_eq!(vec![abyssal, infernal, mortal], hand(&g));

        g.move_card_to_index(mortal, 0);
        assert_eq!(3, hand_key_count(&g));
        assert_eq!(vec![mortal, abyssal, infernal], hand(&g));

        g.move_card_to_index(abyssal, 1);
        assert_eq!(3, hand_key_count(&g));
        assert_eq!(vec![mortal, abyssal, infernal], hand(&g));

        g.move_card_to_index(abyssal, 2);
        assert_eq!(3, hand_key_count(&g));
        assert_eq!(vec![mortal, infernal, abyssal], hand(&g));

        g.move_card_to_index(abyssal, usize::MAX);
        assert_eq!(3, hand_key_count(&g));
        assert_eq!(vec![mortal, infernal, abyssal], hand(&g));
    }

    fn test_game(overlord: Vec<CardName>, champion: Vec<CardName>) -> GameState {
        GameState::new(
            GameId::new(0),
            Deck {
                side: Side::Overlord,
                owner_id: PlayerId::Named(NamedPlayer::TestNoAction),
                identity: CardName::TestOverlordIdentity,
                cards: overlord.into_iter().map(|name| (name, 1)).collect(),
            },
            Deck {
                side: Side::Champion,
                owner_id: PlayerId::Named(NamedPlayer::TestNoAction),
                identity: CardName::TestOverlordIdentity,
                cards: champion.into_iter().map(|name| (name, 1)).collect(),
            },
            GameConfiguration { deterministic: true, ..GameConfiguration::default() },
        )
    }
}
