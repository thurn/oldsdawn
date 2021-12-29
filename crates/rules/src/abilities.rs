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

use data::card_definition::{Ability, AbilityText, AbilityType, Keyword};
use data::card_state::CardPosition;
use data::delegates::{Delegate, EventDelegate, QueryDelegate, Scope};
use data::game::GameState;
use data::primitives::{AttackValue, CardId, ManaValue, Side};

use crate::helpers::*;
use crate::mutations::{move_card, set_raid_ended};
use crate::{mutations, queries};

/// Applies this card's `attack_boost` stat a number of times equal to its
/// [CardState::boost_count]
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

/// The standard weapon ability; applies an attack boost for the duration of a
/// single encounter.
pub fn encounter_boost() -> Ability {
    Ability {
        text: AbilityText::TextFn(|g, s| {
            let boost = queries::stats(g, s).attack_boost.expect("attack_boost");
            vec![mana_cost_text(boost.cost), add_number(boost.bonus), text("Attack")]
        }),
        ability_type: AbilityType::Encounter,
        delegates: vec![
            Delegate::ActivateBoost(EventDelegate::new(this_boost, mutations::write_boost)),
            Delegate::AttackValue(QueryDelegate::new(this_card, add_boost)),
            Delegate::EncounterEnd(EventDelegate::new(always, mutations::clear_boost)),
        ],
    }
}

/// Store N mana in this card when played. Move it to the discard pile when the
/// stored mana is depleted.
pub fn store_mana<const N: ManaValue>() -> Ability {
    Ability {
        text: AbilityText::Text(vec![keyword(Keyword::Play), keyword(Keyword::Store(N))]),
        ability_type: AbilityType::Standard,
        delegates: vec![
            Delegate::PlayCard(EventDelegate::new(this_card, |g, _s, card_id| {
                g.card_mut(card_id).data.stored_mana = N;
            })),
            Delegate::StoredManaTaken(EventDelegate::new(this_card, |g, s, card_id| {
                if g.card(card_id).data.stored_mana == 0 {
                    move_card(g, card_id, CardPosition::DiscardPile(s.side()))
                }
            })),
        ],
    }
}

/// Discard a random card from the hand of the `side` player, if there are any
/// cards present.
pub fn discard_random_card(game: &mut GameState, side: Side) {
    if let Some(card_id) = game.random_card(CardPosition::Hand(side)) {
        move_card(game, card_id, CardPosition::DiscardPile(side));
    }
}

pub fn strike<const N: u32>() -> Ability {
    combat(
        AbilityText::Text(vec![keyword(Keyword::Combat), keyword(Keyword::Strike(N))]),
        |g, _, _| {
            for _ in 0..N {
                discard_random_card(g, Side::Champion);
            }
        },
    )
}

pub fn end_raid() -> Ability {
    combat(AbilityText::Text(vec![keyword(Keyword::Combat), text("End the raid.")]), |g, _, _| {
        set_raid_ended(g);
    })
}
