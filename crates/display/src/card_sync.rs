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

use anyhow::Result;
use data::card_definition::{AbilityType, CardDefinition, TargetRequirement};
use data::card_state::CardState;
use data::game::GameState;
use data::game_actions::CardTarget;
use data::primitives::{AbilityId, ManaValue, RoomId};
use enum_iterator::IntoEnumIterator;
use protos::spelldawn::card_targeting::Targeting;
use protos::spelldawn::{
    ArrowTargetRoom, CardIcon, CardIcons, CardPrefab, CardTargeting, CardTitle, CardView,
    NoTargeting, RevealedCardView, RulesText, TargetingArrow,
};
use rules::{flags, queries};

use crate::assets::CardIconType;
use crate::response_builder::ResponseBuilder;
use crate::{adapters, assets, positions, rules_text};

pub fn card_view(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
) -> Result<CardView> {
    let definition = rules::get(card.name);
    let revealed = card.is_revealed_to(builder.user_side);
    Ok(CardView {
        card_id: Some(adapters::card_identifier(card.id)),
        card_position: Some(positions::convert(builder, game, card)?),
        prefab: CardPrefab::Standard.into(),
        revealed_to_viewer: card.is_revealed_to(builder.user_side),
        is_face_up: card.is_face_up(),
        card_icons: Some(card_icons(game, card, definition, revealed)),
        arena_frame: Some(assets::arena_frame(
            definition.side,
            definition.card_type,
            definition.config.faction,
        )),
        owning_player: builder.to_player_name(definition.side),
        revealed_card: revealed.then(|| revealed_card_view(builder, game, card)),
        create_position: if builder.animate {
            Some(positions::for_card(card, positions::deck_top(builder, card.side())))
        } else {
            None
        },
        destroy_position: if builder.animate {
            Some(positions::for_card(card, positions::deck_top(builder, card.side())))
        } else {
            None
        },
    })
}

pub fn ability_cards(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
) -> Vec<Result<CardView>> {
    let mut result = vec![];
    for (ability_index, ability) in rules::get(card.name).abilities.iter().enumerate() {
        if let AbilityType::Activated(_, target_requirement) = &ability.ability_type {
            let ability_id = AbilityId::new(card.id, ability_index);
            if !flags::activated_ability_has_valid_targets(game, builder.user_side, ability_id) {
                continue;
            }
            result.push(Ok(ability_card_view(builder, game, ability_id, Some(target_requirement))));
        }
    }
    result
}

pub fn ability_card_view(
    builder: &ResponseBuilder,
    game: &GameState,
    ability_id: AbilityId,
    target_requirement: Option<&TargetRequirement<AbilityId>>,
) -> CardView {
    CardView {
        card_id: Some(adapters::ability_card_identifier(ability_id)),
        card_position: Some(positions::for_ability(
            game,
            ability_id,
            positions::hand(builder, ability_id.side()),
        )),
        prefab: CardPrefab::TokenCard.into(),
        revealed_to_viewer: true,
        is_face_up: false,
        card_icons: Some(CardIcons {
            top_left_icon: queries::ability_mana_cost(game, ability_id).map(mana_card_icon),
            ..CardIcons::default()
        }),
        arena_frame: None,
        owning_player: builder.to_player_name(ability_id.card_id.side),
        revealed_card: Some(revealed_ability_card_view(
            builder,
            game,
            ability_id,
            target_requirement,
        )),
        create_position: if builder.animate {
            Some(positions::for_ability(game, ability_id, positions::parent_card(ability_id)))
        } else {
            None
        },
        destroy_position: if builder.animate {
            Some(positions::for_ability(game, ability_id, positions::parent_card(ability_id)))
        } else {
            None
        },
    }
}

fn revealed_card_view(
    builder: &ResponseBuilder,
    game: &GameState,
    card: &CardState,
) -> RevealedCardView {
    let definition = rules::get(card.name);
    RevealedCardView {
        card_frame: Some(assets::card_frame(definition.school)),
        title_background: Some(assets::title_background(definition.config.faction)),
        jewel: Some(assets::jewel(definition.rarity)),
        image: Some(adapters::sprite(&definition.image)),
        title: Some(CardTitle { text: definition.name.displayed_name() }),
        rules_text: Some(rules_text::build(game, card, definition)),
        targeting: Some(card_targeting(definition.config.custom_targeting.as_ref(), |target| {
            flags::can_take_play_card_action(game, builder.user_side, card.id, target)
        })),
        on_release_position: Some(positions::for_card(card, positions::staging())),
        supplemental_info: Some(rules_text::build_supplemental_info(game, card, None)),
    }
}

fn revealed_ability_card_view(
    _builder: &ResponseBuilder,
    game: &GameState,
    ability_id: AbilityId,
    target_requirement: Option<&TargetRequirement<AbilityId>>,
) -> RevealedCardView {
    let card = game.card(ability_id.card_id);
    let definition = rules::get(card.name);
    let ability = definition.ability(ability_id.index);
    RevealedCardView {
        card_frame: Some(assets::ability_card_frame(ability_id.side())),
        title_background: Some(assets::title_background(None)),
        jewel: None,
        image: Some(adapters::sprite(&definition.image)),
        title: Some(CardTitle { text: definition.name.displayed_name() }),
        rules_text: Some(RulesText { text: rules_text::ability_text(game, ability_id, ability) }),
        targeting: Some(card_targeting(target_requirement, |target| {
            flags::can_take_activate_ability_action(game, ability_id.side(), ability_id, target)
        })),
        on_release_position: Some(positions::for_ability(game, ability_id, positions::staging())),
        supplemental_info: Some(rules_text::build_supplemental_info(
            game,
            card,
            Some(ability_id.index),
        )),
    }
}

fn card_targeting<T>(
    requirement: Option<&TargetRequirement<T>>,
    can_play: impl Fn(CardTarget) -> bool,
) -> CardTargeting {
    CardTargeting {
        targeting: Some(match requirement {
            None | Some(TargetRequirement::None) => {
                Targeting::NoTargeting(NoTargeting { can_play: can_play(CardTarget::None) })
            }
            Some(TargetRequirement::TargetRoom(_)) => Targeting::ArrowTargetRoom(ArrowTargetRoom {
                valid_rooms: RoomId::into_enum_iter()
                    .filter(|room_id| can_play(CardTarget::Room(*room_id)))
                    .map(adapters::room_identifier)
                    .collect(),
                arrow: TargetingArrow::Red.into(),
            }),
        }),
    }
}

fn card_icons(
    game: &GameState,
    card: &CardState,
    definition: &CardDefinition,
    revealed: bool,
) -> CardIcons {
    let mut icons = CardIcons::default();

    if card.data.card_level > 0 {
        icons.arena_icon = Some(CardIcon {
            enabled: true,
            background: Some(assets::card_icon(CardIconType::LevelCounter)),
            text: Some(card.data.card_level.to_string()),
            background_scale: assets::background_scale(CardIconType::LevelCounter),
        });
    }

    if card.data.stored_mana > 0 {
        icons.arena_icon = Some(CardIcon {
            enabled: true,
            background: Some(assets::card_icon(CardIconType::Mana)),
            text: Some(card.data.stored_mana.to_string()),
            background_scale: assets::background_scale(CardIconType::Mana),
        });
    }

    if revealed {
        icons.top_left_icon = queries::mana_cost(game, card.id).map(mana_card_icon);
        icons.bottom_right_icon = definition
            .config
            .stats
            .base_attack
            .map(|_| CardIcon {
                enabled: true,
                background: Some(assets::card_icon(CardIconType::Attack)),
                text: Some(queries::attack(game, card.id).to_string()),
                background_scale: assets::background_scale(CardIconType::Attack),
            })
            .or_else(|| {
                definition.config.stats.health.map(|_| CardIcon {
                    enabled: true,
                    background: Some(assets::card_icon(CardIconType::Health)),
                    text: Some(queries::health(game, card.id).to_string()),
                    background_scale: assets::background_scale(CardIconType::Health),
                })
            });
        let shield = queries::shield(game, card.id);
        if shield > 0 {
            icons.bottom_left_icon = Some(CardIcon {
                enabled: true,
                background: Some(assets::card_icon(CardIconType::Shield)),
                text: Some(shield.to_string()),
                background_scale: assets::background_scale(CardIconType::Shield),
            });
        }
    }

    icons
}

fn mana_card_icon(value: ManaValue) -> CardIcon {
    CardIcon {
        enabled: true,
        background: Some(assets::card_icon(CardIconType::Mana)),
        text: Some(value.to_string()),
        background_scale: assets::background_scale(CardIconType::Mana),
    }
}
