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

use data::card_definition::{Ability, AbilityType, Cost};
use data::card_state::CardPosition;
use data::delegates::{Delegate, EventDelegate, QueryDelegate, Scope};
use data::game::GameState;
use data::primitives::{AttackValue, CardId, DamageTypeTrait, ManaValue, Side};
use data::text::{AbilityText, Keyword, TextToken};

use crate::card_text::text;
use crate::helpers::*;
use crate::{mutations, queries};

/// The standard weapon ability; applies an attack boost for the duration of a
/// single encounter.
pub fn encounter_boost() -> Ability {
    Ability {
        text: AbilityText::TextFn(|g, s| {
            let boost = queries::stats(g, s.card_id()).attack_boost.expect("attack_boost");
            vec![
                cost(boost.cost).into(),
                add_number(boost.bonus),
                TextToken::Literal("Attack".to_owned()),
            ]
        }),
        ability_type: AbilityType::Encounter,
        delegates: vec![
            Delegate::ActivateBoost(EventDelegate::new(this_boost, mutations::write_boost)),
            Delegate::AttackValue(QueryDelegate::new(this_card, add_boost)),
            Delegate::EncounterEnd(EventDelegate::new(always, mutations::clear_boost)),
        ],
    }
}

/// Store `N` mana in this card when played. Move it to the discard pile when
/// the stored mana is depleted.
pub fn store_mana<const N: ManaValue>() -> Ability {
    Ability {
        text: text![Keyword::Play, Keyword::Store(N)],
        ability_type: AbilityType::Standard,
        delegates: vec![
            Delegate::PlayCard(EventDelegate::new(this_card, |g, _s, card_id| {
                g.card_mut(card_id).data.stored_mana = N;
            })),
            Delegate::StoredManaTaken(EventDelegate::new(this_card, |g, s, card_id| {
                if g.card(card_id).data.stored_mana == 0 {
                    mutations::move_card(g, card_id, CardPosition::DiscardPile(s.side()))
                }
            })),
        ],
    }
}

/// Activated ability to take `N` stored mana from this card by paying a
pub fn take_mana<const N: ManaValue>(cost: Cost) -> Ability {
    Ability {
        text: text![Keyword::Take(N)],
        ability_type: AbilityType::Activated(cost),
        delegates: vec![Delegate::ActivateAbility(EventDelegate::new(
            this_ability,
            |g, _s, ability_id| {
                assert!(g.card(ability_id.card_id).data.stored_mana >= N);
                g.card_mut(ability_id.card_id).data.stored_mana -= N;
            },
        ))],
    }
}

/// Discard a random card from the hand of the `side` player, if there are any
/// cards present. Invokes the `on_empty` function if a card cannot be
/// discarded.
pub fn discard_random_card(game: &mut GameState, side: Side, on_empty: impl Fn(&mut GameState)) {
    if let Some(card_id) = game.random_card(CardPosition::Hand(side)) {
        mutations::move_card(game, card_id, CardPosition::DiscardPile(side));
    } else {
        on_empty(game);
    }
}

/// Minion combat ability which deals damage to the Champion player during
/// combat, causing them to discard `N` random cards and lose the game if they
/// cannot.
pub fn deal_damage<TDamage: DamageTypeTrait, const N: u32>() -> Ability {
    combat(text![Keyword::Combat, Keyword::DealDamage(N, TDamage::damage_type())], |g, _, _| {
        for _ in 0..N {
            discard_random_card(g, Side::Champion, |g| {
                panic!("Game Over {:?} with {:?}", g.id, TDamage::damage_type())
            });
        }
    })
}

/// Minion combat ability which ends the current raid.
pub fn end_raid() -> Ability {
    combat(text![Keyword::Combat, Keyword::EndRaid], |g, _, _| {
        mutations::end_raid(g);
    })
}

/// Applies this card's `attack_boost` stat a number of times equal to its
/// [CardState::boost_count]. Panics if this card has no attack boost defined.
fn add_boost(
    game: &GameState,
    _scope: Scope,
    card_id: CardId,
    current: AttackValue,
) -> AttackValue {
    let boost_count = queries::boost_count(game, card_id);
    let bonus = queries::stats(game, card_id).attack_boost.expect("Expected boost").bonus;
    current + (boost_count * bonus)
}
