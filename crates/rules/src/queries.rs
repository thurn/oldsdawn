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

//! Core functions for querying the current state of a game

use anyhow::Result;
use data::card_definition::{AbilityType, AttackBoost, CardStats, TargetRequirement};
use data::card_state::{CardPosition, CardState};
use data::delegates::{
    AbilityManaCostQuery, ActionCostQuery, AttackBoostQuery, AttackValueQuery, BoostCountQuery,
    BreachValueQuery, HealthValueQuery, ManaCostQuery, MaximumHandSizeQuery,
    SanctumAccessCountQuery, ShieldValueQuery, StartOfTurnActionsQuery, VaultAccessCountQuery,
};
use data::game::GameState;
use data::game_actions::{CardTarget, CardTargetKind};
use data::primitives::{
    AbilityId, ActionCount, AttackValue, BoostCount, BreachValue, CardId, CardType, HealthValue,
    ItemLocation, ManaValue, RoomId, RoomLocation, ShieldValue, Side,
};

use crate::{constants, dispatch};

/// Obtain the [CardStats] for a given card
pub fn stats(game: &GameState, card_id: CardId) -> &CardStats {
    &crate::get(game.card(card_id).name).config.stats
}

/// Returns the mana cost for a given card.
///
/// - For minions, this is the summon cost.
/// - For projects, this is the unveil cost.
/// - For spells, artifacts, and weapons this is the casting cost.
/// - Schemes do not have a mana cost
pub fn mana_cost(game: &GameState, card_id: CardId) -> Option<ManaValue> {
    dispatch::perform_query(
        game,
        ManaCostQuery(card_id),
        crate::get(game.card(card_id).name).cost.mana,
    )
}

/// Returns the mana cost for a given ability, if any
pub fn ability_mana_cost(game: &GameState, ability_id: AbilityId) -> Option<ManaValue> {
    let cost = if let AbilityType::Activated(cost, _) =
        &crate::get(game.card(ability_id.card_id).name).ability(ability_id.index).ability_type
    {
        cost.mana
    } else {
        None
    };

    dispatch::perform_query(game, AbilityManaCostQuery(ability_id), cost)
}

/// Returns the action point cost for a given card
pub fn action_cost(game: &GameState, card_id: CardId) -> ActionCount {
    dispatch::perform_query(
        game,
        ActionCostQuery(card_id),
        crate::get(game.card(card_id).name).cost.actions,
    )
}

/// Returns the attack power value for a given card, or 0 by default.
pub fn attack(game: &GameState, card_id: CardId) -> AttackValue {
    dispatch::perform_query(
        game,
        AttackValueQuery(card_id),
        stats(game, card_id).base_attack.unwrap_or(0),
    )
}

/// Returns the health value for a given card, or 0 by default.
pub fn health(game: &GameState, card_id: CardId) -> HealthValue {
    dispatch::perform_query(
        game,
        HealthValueQuery(card_id),
        stats(game, card_id).health.unwrap_or(0),
    )
}

/// Returns the shield value for a given card, or 0 by default.
pub fn shield(game: &GameState, card_id: CardId) -> ShieldValue {
    dispatch::perform_query(
        game,
        ShieldValueQuery(card_id),
        stats(game, card_id).shield.unwrap_or(0),
    )
}

/// Returns the breach value for a given card, or 0 by default.
pub fn breach(game: &GameState, card_id: CardId) -> BreachValue {
    dispatch::perform_query(
        game,
        BreachValueQuery(card_id),
        stats(game, card_id).breach.unwrap_or(0),
    )
}

/// Returns the [AttackBoost] for a given card, if any
pub fn attack_boost(game: &GameState, card_id: CardId) -> Option<AttackBoost> {
    crate::card_definition(game, card_id)
        .config
        .stats
        .attack_boost
        .map(|boost| dispatch::perform_query(game, AttackBoostQuery(card_id), boost))
}

/// Returns the [BoostCount] for a given card.
pub fn boost_count(game: &GameState, card_id: CardId) -> BoostCount {
    dispatch::perform_query(game, BoostCountQuery(card_id), game.card(card_id).data.boost_count)
}

/// Returns the amount of mana the owner of `card_id` would need to spend to
/// raise its [AttackValue] to the provided `target` by activating boosts or
/// by using other innate abilities, plus the amount of mana required to pay
/// the shield cost of `target`.
///
/// - Returns 0 if this card can already defeat the target.
/// - Returns None if it is impossible for this card to defeat the target.
pub fn cost_to_defeat_target(
    game: &GameState,
    card_id: CardId,
    target_id: CardId,
) -> Option<ManaValue> {
    let target = health(game, target_id);
    let current = attack(game, card_id);

    let result = if current >= target {
        Some(0)
    } else if let Some(boost) = attack_boost(game, card_id) {
        if boost.bonus == 0 {
            None
        } else {
            let increase = target - current;
            // If the boost does not evenly divide into the target, we need to apply it an
            // additional time.
            let add = if (increase % boost.bonus) == 0 { 0 } else { 1 };

            #[allow(clippy::integer_division)] // Deliberate integer truncation
            Some((add + (increase / boost.bonus)) * boost.cost)
        }
    } else {
        None
    };

    result.map(|r| r + (shield(game, target_id).saturating_sub(breach(game, card_id))))
}

/// Look up the number of action points a player receives at the start of their
/// turn
pub fn start_of_turn_action_count(game: &GameState, side: Side) -> ActionCount {
    dispatch::perform_query(game, StartOfTurnActionsQuery(side), 3)
}

/// Look up the number of cards the Champion player can access from the Vault
/// during the current raid
pub fn vault_access_count(game: &GameState) -> Result<u32> {
    let raid_id = game.raid()?.raid_id;
    Ok(dispatch::perform_query(game, VaultAccessCountQuery(raid_id), 1))
}

/// Look up the number of cards the Champion player can access from the Sanctum
/// during the current raid
pub fn sanctum_access_count(game: &GameState) -> Result<u32> {
    let raid_id = game.raid()?.raid_id;
    Ok(dispatch::perform_query(game, SanctumAccessCountQuery(raid_id), 1))
}

/// Looks up what type of target a given card requires
pub fn card_target_kind(game: &GameState, card_id: CardId) -> CardTargetKind {
    let definition = crate::card_definition(game, card_id);
    if let Some(targeting) = &definition.config.custom_targeting {
        return match targeting {
            TargetRequirement::None => CardTargetKind::None,
            TargetRequirement::TargetRoom(_) => CardTargetKind::Room,
        };
    }

    match definition.card_type {
        CardType::Minion | CardType::Project | CardType::Scheme => CardTargetKind::Room,
        _ => CardTargetKind::None,
    }
}

/// Returns the highest mana cost card among those in the provided
/// `card_iterator` (breaking ties based on sorting key), or None if there is no
/// such card.
pub fn highest_cost<'a>(card_iterator: impl Iterator<Item = &'a CardState>) -> Option<CardId> {
    let cards = card_iterator.collect::<Vec<_>>();
    let max = cards.iter().filter_map(|c| crate::get(c.name).cost.mana).max();
    let mut filtered =
        cards.into_iter().filter(|c| crate::get(c.name).cost.mana == max).collect::<Vec<_>>();
    filtered.sort();
    filtered.first().map(|c| c.id)
}

/// Queries the maximum hand size for a player.
pub fn maximum_hand_size(game: &GameState, side: Side) -> u32 {
    dispatch::perform_query(game, MaximumHandSizeQuery(side), constants::STARTING_MAXIMUM_HAND_SIZE)
}

/// Locates a minion in play, returning its current room and index position
/// within that room, if any.
pub fn minion_position(game: &GameState, minion_id: CardId) -> Option<(RoomId, usize)> {
    match game.card(minion_id).position() {
        CardPosition::Room(room_id, location) if location == RoomLocation::Defender => {
            let index = game.defender_list(room_id).iter().position(|cid| *cid == minion_id);
            index.map(|i| (room_id, i))
        }
        _ => None,
    }
}

/// Returns the position to which a card should be moved after being played by
/// the [Side] player with a given [CardTarget].
pub fn played_position(
    game: &GameState,
    side: Side,
    card_id: CardId,
    target: CardTarget,
) -> Result<CardPosition> {
    Ok(match crate::card_definition(game, card_id).card_type {
        CardType::ChampionSpell | CardType::OverlordSpell => CardPosition::DiscardPile(side),
        CardType::Weapon => CardPosition::ArenaItem(ItemLocation::Weapons),
        CardType::Artifact => CardPosition::ArenaItem(ItemLocation::Artifacts),
        CardType::Minion => CardPosition::Room(target.room_id()?, RoomLocation::Defender),
        CardType::Project | CardType::Scheme => {
            CardPosition::Room(target.room_id()?, RoomLocation::Occupant)
        }
        CardType::Identity => CardPosition::Identity(side),
    })
}
