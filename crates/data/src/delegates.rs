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

//! 'Delegates' are the core abstraction of the Spelldawn rules engine.
//!
//! There are two types of delegates: 'Events' and 'Queries'. Event delegates
//! allow cards to respond to specific events which occur during a game, such as
//! taking an action when a card is played or at the start of a turn.
//!
//! Query delegates allow cards to read & intercept requests for game data --
//! for example, the 'can play card' query is used to determine whether a card
//! can be legally played, a card delegate might add custom logic to determine
//! when it can be played. Similarly, the 'attack value' query is used to
//! determine the attack strength of a weapon; a delegate could intercept this
//! request to change the attack power of a given card.
//!
//! Every delegate in the game is run for every applicable event. Even when
//! cards are shuffled into a player's deck, their delegates are invoked. Each
//! delegate has a [RequirementFn] which needs to return true when the delegate
//! should run.
//!
//! Currently, Overlord delegates ares always invoked before Champion delegates,
//! and they are called in alphabetical order by card name.
//!
//! Delegate enum members automatically have an associated struct generated for
//! them by the [DelegateEnum] macro, which is the name of the enum variant with
//! the prefix `Event` or `Query`, e.g. [DawnEvent] for `Delegate::Dawn`.
//!
//! # Example Generated Code
//! We generate approximately the following code for each delegate enum value:
//!
//! ```
//! #[derive(Debug, Copy, Clone)]
//! pub struct OnDawnEvent(pub TurnNumber);
//!
//! impl EventData<TurnNumber> for OnDawnEvent {
//!     fn data(&self) -> TurnNumber {
//!         self.0
//!     }
//!
//!     fn extract(delegate: &Delegate) -> Option<EventDelegate<TurnNumber>> {
//!         match delegate {
//!             Delegate::OnDawn(d) => Some(*d),
//!             _ => None,
//!         }
//!     }
//! }
//! ```

#![allow(clippy::use_self)] // Required to use EnumKind

use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;

use anyhow::Result;
use enum_kinds::EnumKind;
use macros::DelegateEnum;

use crate::card_definition::AttackBoost;
#[allow(unused)] // Used in rustdocs
use crate::card_definition::CardStats;
#[allow(unused)] // Used in rustdocs
use crate::card_definition::Cost;
#[allow(unused)] // Used in rustdocs
use crate::card_state::{CardData, CardPosition};
use crate::game::GameState;
use crate::game_actions::{CardPromptAction, CardTarget, PromptAction};
use crate::primitives::{
    AbilityId, ActionCount, AttackValue, BoostCount, BoostData, BreachValue, CardId, DamageType,
    HasAbilityId, HasCardId, HealthValue, ManaValue, RaidId, RoomId, ShieldValue, Side, TurnNumber,
};

/// Identifies the context for a given request to a delegate: which player,
/// card, & card ability owns the delegate.
#[derive(PartialEq, Eq, Hash, Copy, Clone)]
pub struct Scope {
    /// Ability which owns this delegate.
    ability_id: AbilityId,
}

impl Scope {
    pub fn new(ability_id: AbilityId) -> Self {
        Self { ability_id }
    }

    /// Player who owns this scope
    pub fn side(&self) -> Side {
        self.card_id().side
    }

    /// Ability which owns this scope
    pub fn ability_id(&self) -> AbilityId {
        self.ability_id
    }

    /// Card which owns this scope
    pub fn card_id(&self) -> CardId {
        self.ability_id.card_id
    }
}

impl fmt::Debug for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.ability_id)
    }
}

impl HasAbilityId for Scope {
    fn ability_id(&self) -> AbilityId {
        self.ability_id
    }
}

/// Predicate to determine whether a delegate should run, taking contextual
/// information `T`.
pub type RequirementFn<T> = fn(&GameState, Scope, &T) -> bool;
/// Function to mutate game state in response to an event, taking contextual
/// information `T`.
pub type MutationFn<T> = fn(&mut GameState, Scope, &T) -> Result<()>;
/// Function to intercept a query for game information, taking contextual
/// information `T` and the current query value `R`.
pub type TransformationFn<T, R> = fn(&GameState, Scope, &T, R) -> R;

/// Delegate which responds to a given game event and mutates game state in
/// response.
#[derive(Copy, Clone)]
pub struct EventDelegate<T> {
    /// Should return true if this delegate's `mutation`.
    pub requirement: RequirementFn<T>,
    /// Modifies the current [GameState] in response to the associated event.
    pub mutation: MutationFn<T>,
}

impl<T> EventDelegate<T> {
    pub fn new(requirement: RequirementFn<T>, mutation: MutationFn<T>) -> Self {
        Self { requirement, mutation }
    }
}

/// Delegate which intercepts and transforms a query for game information.
#[derive(Copy, Clone)]
pub struct QueryDelegate<T, R> {
    /// Should return true if this delegate's `transformation` should run.
    pub requirement: RequirementFn<T>,
    /// Function which takes contextual data and the current value of some piece
    /// of game information and returns a transformed value for this
    /// information.
    pub transformation: TransformationFn<T, R>,
}

impl<T, R> QueryDelegate<T, R> {
    pub fn new(requirement: RequirementFn<T>, transformation: TransformationFn<T, R>) -> Self {
        Self { requirement, transformation }
    }
}

/// A Flag is a variant of boolean which typically indicates whether some game
/// action can currently be taken. Flags have a 'default' state, which is the
/// value of the flag based on standard game rules, and an 'override' state,
/// which is a value set by specific delegates. An override of 'false' takes
/// precedence over an override of 'true'.
///
/// For example, the 'CanPlay' delegate will be invoked with
/// `Flag::Default(false)` if a card cannot currently be played according to the
/// standard game rules (sufficient mana available, correct player's turn, etc).
/// A delegate could transform this via `with_override(true)` to allow the card
/// to be played. A second delegate could set `with_override(false)` to prevent
/// the card from being played, and this would take priority.
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum Flag {
    /// Initial value of this flag
    Default(bool),
    /// Override for this flag set by a delegate
    Override(bool),
}

impl Flag {
    pub fn new(value: bool) -> Self {
        Self::Default(value)
    }

    /// Incorporates an override into this flag, following the precedence rules
    /// described for [Flag] above.
    pub fn with_override(self, value: bool) -> Self {
        match self {
            Self::Default(_) => Self::Override(value),
            Self::Override(current) => Self::Override(current && value),
        }
    }
}

impl From<Flag> for bool {
    fn from(flag: Flag) -> Self {
        match flag {
            Flag::Default(value) | Flag::Override(value) => value,
        }
    }
}

/// Event data for when a card is played
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct CardPlayed {
    pub card_id: CardId,
    pub target: CardTarget,
}

impl HasCardId for CardPlayed {
    fn card_id(&self) -> CardId {
        self.card_id
    }
}

/// Event data for when an ability is activated
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct AbilityActivated {
    pub ability_id: AbilityId,
    pub target: CardTarget,
}

impl AbilityActivated {
    pub fn card_id(&self) -> CardId {
        self.ability_id.card_id
    }
}

impl HasAbilityId for AbilityActivated {
    fn ability_id(&self) -> AbilityId {
        self.ability_id
    }
}

/// Event data for when a card is moved
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct CardMoved {
    /// Position before the move
    pub old_position: CardPosition,
    /// New card position, where the the card is now located.
    pub new_position: CardPosition,
}

/// Event data for encounters between cards
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct CardEncounter {
    /// Card initiating the interaction
    pub source: CardId,
    /// Card being targeted
    pub target: CardId,
}

impl CardEncounter {
    pub fn new(source: CardId, target: CardId) -> Self {
        Self { source, target }
    }
}

/// Event data when a raid is initiated
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct RaidStart {
    pub raid_id: RaidId,
    pub target: RoomId,
}

/// Event data when a weapon is used
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct UsedWeapon {
    pub raid_id: RaidId,
    pub weapon_id: CardId,
    pub target_id: CardId,
    pub mana_spent: ManaValue,
}

/// Event data when a card is scored
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct ScoreCard {
    pub player: Side,
    pub card_id: CardId,
}

impl HasCardId for ScoreCard {
    fn card_id(&self) -> CardId {
        self.card_id
    }
}

/// Result of a raid
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub enum RaidOutcome {
    Success,
    Failure,
}

/// Event data when a raid is completed
#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct RaidEnded {
    pub raid_id: RaidId,
    pub outcome: RaidOutcome,
}

impl From<RaidEnded> for RaidId {
    fn from(this: RaidEnded) -> Self {
        this.raid_id
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub struct DealtDamage {
    pub source: AbilityId,
    pub amount: u32,
    pub damage_type: DamageType,
    pub discarded: Vec<CardId>,
}

impl HasAbilityId for DealtDamage {
    fn ability_id(&self) -> AbilityId {
        self.source
    }
}

/// Actions to show the Champion during combat in addition to their weapon
/// actions
#[derive(Clone, Debug)]
pub struct MinionCombatPrompt {
    /// Combat actions to show the Champion
    pub actions: Vec<PromptAction>,
    /// Whether to show the default continue/don't use a weapon option
    pub include_no_action: bool,
}

/// The core of the delegate pattern, used to identify which event or which
/// query this delegate wishes to respond to. Each enum variant here
/// automatically gets an associated struct value generated for it by the
/// [DelegateEnum] macro -- see module-level documentation for an example of
/// what this code looks like.
#[derive(EnumKind, DelegateEnum, Clone)]
#[enum_kind(DelegateKind, derive(Hash))]
pub enum Delegate {
    /// The Champion's turn begins
    Dawn(EventDelegate<TurnNumber>),
    /// The Overlord's turn begins
    Dusk(EventDelegate<TurnNumber>),
    /// A card is moved from a Deck position to a Hand position
    DrawCard(EventDelegate<CardId>),
    /// A user takes the explicit 'draw card' game action
    DrawCardAction(EventDelegate<CardId>),
    /// A card has been selected to play via the Play action and should have
    /// additional costs deducted.
    PayCardCosts(EventDelegate<CardId>),
    /// A card has been played via the Play Card action and has had its costs
    /// paid
    CastCard(EventDelegate<CardPlayed>),
    /// A card has been moved from any non-arena zone to an arena zone.
    EnterPlay(EventDelegate<CardId>),
    /// A card ability with a cost is activated
    ActivateAbility(EventDelegate<AbilityActivated>),
    /// A project card is unveiled (turned face up by paying its cost)
    UnveilProject(EventDelegate<CardId>),
    /// A minion card is turned face up.
    SummonMinion(EventDelegate<CardId>),
    /// A card is moved to a new position
    MoveCard(EventDelegate<CardMoved>),
    /// A card is scored by the Overlord
    OverlordScoreCard(EventDelegate<CardId>),
    /// A card is scored by the Champion
    ChampionScoreCard(EventDelegate<CardId>),
    /// Either player scores a card
    ScoreCard(EventDelegate<ScoreCard>),
    /// A Raid is initiated
    RaidStart(EventDelegate<RaidStart>),
    /// A minion is encountered during a raid
    EncounterMinion(EventDelegate<CardId>),
    /// A weapon boost is activated for a given card
    ActivateBoost(EventDelegate<BoostData>),
    /// A weapon has been used to defeat a minion
    UsedWeapon(EventDelegate<UsedWeapon>),
    /// A minion is defeated during an encounter by dealing damage to it equal
    /// to its health
    MinionDefeated(EventDelegate<CardId>),
    /// A minion's 'combat' ability is triggered during an encounter, typically
    /// because the minion was not defeated by the Champion.
    MinionCombatAbility(EventDelegate<CardId>),
    /// A minion finishes being encountered during a raid. Invokes regardless of
    /// whether the encounter was successful.
    EncounterEnd(EventDelegate<RaidId>),
    /// Minion encounters have been completed for a raid and the Access phase is
    /// about to start. The set of accessed cards has not yet been selected.
    RaidAccessStart(EventDelegate<RaidId>),
    /// The card with the provided `card_id` has been accessed during a raid (in
    /// any zone), but not yet scored/acted on.
    CardAccess(EventDelegate<CardId>),
    /// A Raid is completed, either successfully or unsuccessfully.
    ///
    /// Note that this is invoked before `game.data.raid` is cleared.
    RaidEnd(EventDelegate<RaidEnded>),
    /// A raid has ended in failure.
    RaidFailure(EventDelegate<RaidId>),
    /// A raid has ended in success.
    RaidSuccess(EventDelegate<RaidId>),
    /// Stored mana is taken from a card
    StoredManaTaken(EventDelegate<CardId>),
    /// Damage has been dealt to the Champion player (in the form of discarded
    /// cards).
    DealtDamage(EventDelegate<DealtDamage>),

    /// Query whether the indicated player can currently take the basic game
    /// action to spend an action point to draw a card.
    CanTakeDrawCardAction(QueryDelegate<Side, Flag>),
    /// Query whether the indicated player can currently take the basic game
    /// action to spend an action point to gain one mana
    CanTakeGainManaAction(QueryDelegate<Side, Flag>),
    /// Query whether a given card can currently be played.
    CanPlayCard(QueryDelegate<CardId, Flag>),
    /// Query whether a given ability can currently be activated.
    CanActivateAbility(QueryDelegate<AbilityId, Flag>),
    /// Query whether an ability can be activated while its card is face-down
    CanActivateWhileFaceDown(QueryDelegate<AbilityId, Flag>),
    /// Can the indicated player currently take the basic game action to
    /// initiate a raid?
    CanInitiateRaid(QueryDelegate<Side, Flag>),
    /// Can the indicated player currently take the basic game action to level
    /// up a room?
    CanLevelUpRoom(QueryDelegate<Side, Flag>),
    /// Can the indicated card be leveled up when the level up action is taken
    /// for a room?
    ///
    /// Note that Scheme cards can be leveled up by default.
    CanLevelUpCard(QueryDelegate<CardId, Flag>),
    /// Can the source card (typically a weapon) take an encounter action
    /// against the target card (typically a minion) during a raid?
    CanEncounterTarget(QueryDelegate<CardEncounter, Flag>),
    /// Can the source card (typically a weapon) apply an encounter
    /// action to defeat the target target (typically a minion) during a raid?
    CanDefeatTarget(QueryDelegate<CardEncounter, Flag>),

    /// Query the current mana cost of a card. Invoked with [Cost::mana].
    ManaCost(QueryDelegate<CardId, Option<ManaValue>>),
    /// Query the current mana cost of an ability. Invoked with [Cost::mana].
    AbilityManaCost(QueryDelegate<AbilityId, Option<ManaValue>>),
    /// Query the current mana cost of a card. Invoked with [Cost::actions].
    ActionCost(QueryDelegate<CardId, ActionCount>),
    /// Query the current attack value of a card. Invoked with
    /// [CardStats::base_attack] or 0.
    AttackValue(QueryDelegate<CardId, AttackValue>),
    /// Query the current health value of a card. Invoked with
    /// [CardStats::health] or 0.
    HealthValue(QueryDelegate<CardId, HealthValue>),
    /// Query the current shield value of a card. Invoked with
    /// [CardStats::shield] or 0.
    ShieldValue(QueryDelegate<CardId, ShieldValue>),
    /// Queries the current breach value of a card. Invoked with
    /// [CardStats::breach] or 0.
    BreachValue(QueryDelegate<CardId, BreachValue>),
    /// Gets the current [AttackBoost] of a card. Invoked with
    /// [CardStats::attack_boost] if one is present.
    AttackBoost(QueryDelegate<CardId, AttackBoost>),
    /// Get the current boost count of a card. Invoked with the value of
    /// [CardData::boost_count].
    BoostCount(QueryDelegate<CardId, BoostCount>),
    /// Get the number of actions a player gets at the start of their turn.
    StartOfTurnActions(QueryDelegate<Side, ActionCount>),
    /// Gets the number of cards the Champion player can access from the Vault
    /// during this raid
    VaultAccessCount(QueryDelegate<RaidId, u32>),
    /// Gets the number of cards the Champion player can access from the Sanctum
    /// during this raid
    SanctumAccessCount(QueryDelegate<RaidId, u32>),
    /// Queries the maximum hand size of a player. Invoked with the default
    /// maximum hand size.
    MaximumHandSize(QueryDelegate<Side, u32>),
    /// Actions to present when a minion is encountered in combat in addition to
    /// weapon abilities. Invoked with the empty vector. If no actions are
    /// returned, a default 'continue' action is shown.
    MinionCombatActions(QueryDelegate<CardId, Vec<Option<CardPromptAction>>>),
}

impl Delegate {
    pub fn kind(&self) -> DelegateKind {
        self.into()
    }
}

impl fmt::Debug for Delegate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Delegate::{:?}", DelegateKind::from(self))
    }
}

/// Contains the state needed to invoke a delegate within the context of a
/// specific game.
#[derive(Clone, Debug)]
pub struct DelegateContext {
    pub delegate: Delegate,
    pub scope: Scope,
}

/// Caches delegates in a given game for faster lookup
#[derive(Clone, Debug, Default)]
pub struct DelegateCache {
    pub lookup: HashMap<DelegateKind, Vec<DelegateContext>>,
}

impl DelegateCache {
    pub fn delegate_count(&self, kind: DelegateKind) -> usize {
        self.lookup.get(&kind).map_or(0, Vec::len)
    }

    /// Gets the [DelegateContext] for a given [DelegateKind] and index.
    ///
    /// Panics if no such delegate exists.
    pub fn get(&self, kind: DelegateKind, index: usize) -> &DelegateContext {
        &self.lookup.get(&kind).expect("Delegate")[index]
    }
}

/// Functions implemented by an Event struct, automatically implemented by
/// deriving [DelegateEnum]
pub trait EventData<T: fmt::Debug>: fmt::Debug {
    /// Get the underlying data for this event
    fn data(&self) -> &T;

    fn kind(&self) -> DelegateKind;

    /// Return the wrapped [EventDelegate] if the provided [Delegate] is of the
    /// matching type.
    fn extract(delegate: &Delegate) -> Option<&EventDelegate<T>>;
}

/// Functions implemented by a Query struct, automatically implemented by
/// deriving [DelegateEnum]
pub trait QueryData<TData: fmt::Debug, TResult: fmt::Debug>: fmt::Debug {
    /// Get the underlying data for this query
    fn data(&self) -> &TData;

    fn kind(&self) -> DelegateKind;

    /// Return the wrapped [QueryDelegate] if the provided [Delegate] is of the
    /// matching type.
    fn extract(delegate: &Delegate) -> Option<&QueryDelegate<TData, TResult>>;
}
