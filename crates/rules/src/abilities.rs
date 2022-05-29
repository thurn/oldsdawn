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

//! Helpers for defining common card abilities

use data::card_definition::{Ability, AbilityType, Cost, TargetRequirement};
use data::card_state::CardPosition;
use data::delegates::{Delegate, EventDelegate, QueryDelegate, RaidOutcome, Scope};
use data::game::GameState;
use data::primitives::{AbilityId, AttackValue, CardId, DamageTypeTrait, ManaValue};
use data::text::{AbilityText, Keyword, Sentence, TextToken};

use crate::helpers::*;
use crate::mutations::OnZeroStored;
use crate::text_macro::text;
use crate::{mutations, queries};

/// The standard weapon ability; applies an attack boost for the duration of a
/// single encounter.
pub fn encounter_boost() -> Ability {
    Ability {
        text: AbilityText::TextFn(|g, s| {
            let boost = queries::attack_boost(g, s.card_id()).expect("attack_boost");
            vec![
                cost(boost.cost).into(),
                add_number(boost.bonus),
                TextToken::Literal("Attack".to_owned()),
            ]
        }),
        ability_type: AbilityType::Encounter,
        delegates: vec![
            Delegate::ActivateBoost(EventDelegate::new(this_card, mutations::write_boost)),
            Delegate::AttackValue(QueryDelegate::new(this_card, add_boost)),
            Delegate::EncounterEnd(EventDelegate::new(always, mutations::clear_boost)),
        ],
    }
}

/// Store `N` mana in this card when played. Move it to the discard pile when
/// the stored mana is depleted.
pub fn store_mana_on_play<const N: ManaValue>() -> Ability {
    Ability {
        text: text![Keyword::Play, Keyword::Store(Sentence::Start, N)],
        ability_type: AbilityType::Standard,
        delegates: vec![
            Delegate::CastCard(EventDelegate::new(this_card, |g, _s, played| {
                g.card_mut(played.card_id).data.stored_mana = N;
            })),
            Delegate::StoredManaTaken(EventDelegate::new(this_card, |g, s, card_id| {
                if g.card(*card_id).data.stored_mana == 0 {
                    mutations::move_card(g, *card_id, CardPosition::DiscardPile(s.side()))
                }
            })),
        ],
    }
}

/// Activated ability to take `N` stored mana from this card by paying a cost
pub fn activated_take_mana<const N: ManaValue>(cost: Cost<AbilityId>) -> Ability {
    Ability {
        text: text![Keyword::Take(Sentence::Start, N)],
        ability_type: AbilityType::Activated(cost, TargetRequirement::None),
        delegates: vec![on_activated(|g, _s, activated| {
            mutations::take_stored_mana(g, activated.card_id(), N, OnZeroStored::Sacrifice);
        })],
    }
}

/// Minion combat ability which deals damage to the Champion player during
/// combat, causing them to discard `N` random cards and lose the game if they
/// cannot.
pub fn combat_deal_damage<TDamage: DamageTypeTrait, const N: u32>() -> Ability {
    Ability {
        text: text![Keyword::Combat, Keyword::DealDamage(N, TDamage::damage_type())],
        ability_type: AbilityType::Standard,
        delegates: vec![combat(|g, s, _| {
            mutations::deal_damage(g, s, TDamage::damage_type(), N);
        })],
    }
}

/// Minion combat ability which ends the current raid in failure.
pub fn end_raid() -> Ability {
    Ability {
        text: text![Keyword::Combat, "End the raid."],
        ability_type: AbilityType::Standard,
        delegates: vec![combat(|g, _, _| {
            mutations::end_raid(g, RaidOutcome::Failure);
        })],
    }
}

/// Applies this card's `attack_boost` stat a number of times equal to its
/// [CardState::boost_count]. Panics if this card has no attack boost defined.
fn add_boost(game: &GameState, _: Scope, card_id: &CardId, current: AttackValue) -> AttackValue {
    let boost_count = queries::boost_count(game, *card_id);
    let bonus = queries::attack_boost(game, *card_id).expect("Expected boost").bonus;
    current + (boost_count * bonus)
}

/// An ability which allows a card to have level counters placed on it.
pub fn level_up() -> Ability {
    Ability {
        text: text![Keyword::LevelUp],
        ability_type: AbilityType::Standard,
        delegates: vec![Delegate::CanLevelUpCard(QueryDelegate {
            requirement: this_card,
            transformation: |_g, _, _, current| current.with_override(true),
        })],
    }
}

pub fn construct() -> Ability {
    Ability {
        text: text![Keyword::Construct],
        ability_type: AbilityType::Standard,
        delegates: vec![Delegate::MinionDefeated(EventDelegate {
            requirement: this_card,
            mutation: |g, s, _| {
                mutations::move_card(g, s.card_id(), CardPosition::DiscardPile(s.side()));
            },
        })],
    }
}
