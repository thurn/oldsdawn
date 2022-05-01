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

//! Functions for representing the current game state to the user.

use std::collections::BTreeMap;

use data::card_definition::{AbilityType, CardDefinition, CustomTargeting};
use data::card_state::{CardPosition, CardState};
use data::game::{GamePhase, GameState, MulliganData, RaidData, RaidPhase};
use data::game_actions::CardTarget;
use data::primitives::{
    AbilityId, AbilityIndex, CardId, CardType, ItemLocation, RoomId, RoomLocation, Side, Sprite,
};
use enum_iterator::IntoEnumIterator;
use protos::spelldawn::card_targeting::Targeting;
use protos::spelldawn::game_object_identifier::Id;
use protos::spelldawn::object_position::Position;
#[allow(unused)] // Used in rustdoc
use protos::spelldawn::{
    ActionTrackerView, CardCreationAnimation, CardIcon, CardIcons, CardTargeting, CardTitle,
    CardView, ClientItemLocation, ClientRoomLocation, CreateOrUpdateCardCommand, GameIdentifier,
    GameView, ManaView, ObjectPosition, ObjectPositionDeck, ObjectPositionDiscardPile,
    ObjectPositionHand, ObjectPositionIdentity, ObjectPositionItem, ObjectPositionRaid,
    ObjectPositionRoom, ObjectPositionStaging, PlayInRoom, PlayerInfo, PlayerName, PlayerView,
    RenderInterfaceCommand, RevealedCardView, RoomIdentifier, ScoreView, SpriteAddress,
    UpdateGameViewCommand,
};
use protos::spelldawn::{
    ArrowTargetRoom, CardIdentifier, CardPrefab, NoTargeting, ObjectPositionBrowser,
    ObjectPositionIntoCard, RulesText, TargetingArrow,
};
use rules::mana::ManaType;
use rules::{flags, mana, queries};

use crate::assets::CardIconType;
use crate::response_builder::ResponseOptions;
use crate::{adapters, assets, interface, rules_text};

/// State synchronization response, containing commands for the updated state of
/// each game object in an ongoing game.
pub struct FullSync {
    /// Overall game state
    pub game: UpdateGameViewCommand,
    /// The state of each card in this game
    pub cards: BTreeMap<CardIdentifier, CreateOrUpdateCardCommand>,
    /// Content to display in the user interface
    pub interface: RenderInterfaceCommand,
    /// Positions for Game Objects which are in non-standard positions, e.g.
    /// because they are currently participating in a raid.
    pub position_overrides: BTreeMap<Id, ObjectPosition>,
}

/// Builds a complete representation of the provided game as viewed by the
/// `user_side` player. The game state itself is included, as well as a
/// [CreateOrUpdateCardCommand] for each card in the game.
///
/// If [CardCreationStrategy] values are provided in the `card_creation` map,
/// these override the default card creation behavior of placing cards in their
/// current game position.
pub fn run(game: &GameState, user_side: Side, options: ResponseOptions) -> FullSync {
    FullSync {
        game: update_game_view(game, user_side),
        cards: game
            .all_cards()
            .filter(|c| !c.position().shuffled_into_deck())
            .map(|c| {
                (
                    adapters::adapt_card_id(c.id),
                    create_or_update_card(
                        game,
                        c,
                        user_side,
                        CardCreationStrategy::SnapToCurrentPosition,
                    ),
                )
            })
            .chain(
                game.all_cards()
                    .filter(|c| c.position().in_play() && c.side() == user_side)
                    .flat_map(|c| create_ability_cards(game, c, user_side, options)),
            )
            .collect(),
        interface: interface::render(game, user_side),
        position_overrides: position_overrides(game, user_side),
    }
}

/// Builds a command to update the client's current [GameView]
fn update_game_view(game: &GameState, user_side: Side) -> UpdateGameViewCommand {
    UpdateGameViewCommand {
        game: Some(GameView {
            game_id: Some(adapters::adapt_game_id(game.id)),
            user: Some(player_view(game, user_side)),
            opponent: Some(player_view(game, user_side.opponent())),
            raid_active: game.data.raid.is_some(),
        }),
    }
}

/// Builds a [PlayerView] for a given player
fn player_view(game: &GameState, side: Side) -> PlayerView {
    let identity = game.identity(side);
    PlayerView {
        side: adapters::adapt_side(side).into(),
        player_info: Some(PlayerInfo {
            name: Some(identity.name.displayed_name()),
            portrait: Some(sprite(&rules::get(identity.name).image)),
            portrait_frame: Some(assets::identity_card_frame(side)),
            valid_rooms_to_visit: match side {
                Side::Overlord => RoomId::into_enum_iter()
                    .filter(|room_id| flags::can_take_level_up_room_action(game, side, *room_id))
                    .map(|room_id| adapters::adapt_room_id(room_id).into())
                    .collect(),
                Side::Champion => RoomId::into_enum_iter()
                    .filter(|room_id| flags::can_take_initiate_raid_action(game, side, *room_id))
                    .map(|room_id| adapters::adapt_room_id(room_id).into())
                    .collect(),
            },
            card_back: Some(assets::card_back(rules::get(identity.name).school)),
        }),
        score: Some(ScoreView { score: game.player(side).score }),
        mana: Some(ManaView { amount: mana::get(game, side, ManaType::BaseForDisplay) }),
        action_tracker: Some(ActionTrackerView {
            available_action_count: game.player(side).actions,
        }),
        can_take_action: queries::can_take_action(game, side),
    }
}

/// Possible behavior when creating a card -- used to enable different 'appear'
/// animations for the new card.
#[derive(Debug, PartialEq, Clone)]
pub enum CardCreationStrategy {
    /// Animate the card moving from the user's deck to the staging area.
    DrawUserCard,
    /// Jump the newly-created card to its current game position. If the current
    /// position is invalid (e.g. in the user's deck), no initial position
    /// will be specified for the card.
    SnapToCurrentPosition,
    /// Create the card at a specific game object position.
    CreateAtPosition(ObjectPosition),
}

/// Creates a command to create or update a card.
pub fn create_or_update_card(
    game: &GameState,
    card: &CardState,
    user_side: Side,
    creation_strategy: CardCreationStrategy,
) -> CreateOrUpdateCardCommand {
    let create_animation = if creation_strategy == CardCreationStrategy::DrawUserCard {
        CardCreationAnimation::DrawCard.into()
    } else {
        CardCreationAnimation::Unspecified.into()
    };
    let position = match creation_strategy {
        CardCreationStrategy::DrawUserCard => None,
        CardCreationStrategy::SnapToCurrentPosition => adapt_position(card, user_side),
        CardCreationStrategy::CreateAtPosition(p) => Some(p),
    };

    CreateOrUpdateCardCommand {
        card: Some(card_view(game, card, user_side)),
        create_position: position,
        create_animation,
        disable_flip_animation: false,
    }
}

/// Creates token cards representing activated abilities of the provided `card`.
fn create_ability_cards(
    game: &GameState,
    card: &CardState,
    user_side: Side,
    options: ResponseOptions,
) -> Vec<(CardIdentifier, CreateOrUpdateCardCommand)> {
    let mut result = vec![];
    for (ability_index, ability) in rules::get(card.name).abilities.iter().enumerate() {
        if let AbilityType::Activated(_cost) = &ability.ability_type {
            let identifier = adapters::adapt_ability_card_id(card.id, AbilityIndex(ability_index));
            result.push((
                identifier,
                CreateOrUpdateCardCommand {
                    card: Some(ability_card_view(
                        game,
                        AbilityId::new(card.id, ability_index),
                        user_side,
                        true, /* check_can_play */
                    )),
                    create_position: Some(ObjectPosition {
                        sorting_key: card.sorting_key,
                        position: Some(Position::Hand(ObjectPositionHand {
                            owner: PlayerName::User.into(),
                        })),
                    }),
                    create_animation: if options.contains(ResponseOptions::IS_INITIAL_CONNECT) {
                        CardCreationAnimation::Unspecified
                    } else {
                        CardCreationAnimation::FromParentCard
                    }
                    .into(),
                    disable_flip_animation: false,
                },
            ))
        }
    }
    result
}

/// Creates a `CardView` representing an ability of a provided `card`.
pub fn ability_card_view(
    game: &GameState,
    ability_id: AbilityId,
    user_side: Side,
    check_can_play: bool,
) -> CardView {
    let card = game.card(ability_id.card_id);
    let identifier = adapters::adapt_ability_id(ability_id);
    let definition = rules::get(card.name);
    let ability = rules::get(card.name).ability(ability_id.index);
    let mana_cost =
        if let AbilityType::Activated(cost) = &ability.ability_type { cost.mana } else { None };
    CardView {
        card_id: Some(identifier),
        prefab: CardPrefab::TokenCard.into(),
        revealed_to_viewer: true,
        is_face_up: false,
        card_icons: mana_cost.map(|mana| CardIcons {
            top_left_icon: Some(CardIcon {
                background: Some(assets::card_icon(CardIconType::Mana)),
                text: Some(mana.to_string()),
                background_scale: assets::background_scale(CardIconType::Mana),
            }),
            ..CardIcons::default()
        }),
        arena_frame: None,
        owning_player: adapters::to_player_name(card.side(), user_side).into(),
        revealed_card: Some(RevealedCardView {
            card_frame: Some(assets::ability_card_frame(card.side())),
            title_background: Some(assets::title_background(None)),
            jewel: None,
            image: Some(sprite(&definition.image)),
            title: Some(CardTitle { text: format!("Use {}", definition.name.displayed_name()) }),
            rules_text: Some(RulesText {
                text: rules_text::ability_text(game, ability_id, ability),
            }),
            targeting: Some(CardTargeting {
                targeting: Some(Targeting::NoTargeting(NoTargeting {
                    can_play: if check_can_play {
                        flags::can_take_activate_ability_action(game, user_side, ability_id)
                    } else {
                        false
                    },
                })),
            }),
            on_release_position: Some(ObjectPosition {
                sorting_key: card.sorting_key,
                position: Some(Position::IntoCard(ObjectPositionIntoCard {
                    card_id: Some(adapters::adapt_card_id(card.id)),
                })),
            }),
            supplemental_info: Some(rules_text::build_supplemental_info(
                game,
                card,
                Some(ability_id.index),
            )),
        }),
    }
}

/// Converts a [CardState] into a [CardView] for a given game state.
pub fn card_view(game: &GameState, card: &CardState, user_side: Side) -> CardView {
    let definition = rules::get(card.name);
    let revealed = card.is_revealed_to(user_side);
    CardView {
        card_id: Some(adapters::adapt_card_id(card.id)),
        prefab: CardPrefab::Standard.into(),
        revealed_to_viewer: card.is_revealed_to(user_side),
        is_face_up: card.is_face_up(),
        card_icons: Some(card_icons(game, card, definition, revealed)),
        arena_frame: Some(assets::arena_frame(
            definition.side,
            definition.card_type,
            definition.config.faction,
        )),
        owning_player: adapters::to_player_name(definition.side, user_side).into(),
        revealed_card: revealed.then(|| revealed_card_view(game, card, definition, user_side)),
    }
}

/// Build icons struct for this card
fn card_icons(
    game: &GameState,
    card: &CardState,
    definition: &CardDefinition,
    revealed: bool,
) -> CardIcons {
    let mut icons = CardIcons::default();

    if card.data.card_level > 0 {
        icons.arena_icon = Some(CardIcon {
            background: Some(assets::card_icon(CardIconType::LevelCounter)),
            text: Some(card.data.card_level.to_string()),
            background_scale: assets::background_scale(CardIconType::LevelCounter),
        });
    }

    if card.data.stored_mana > 0 {
        icons.arena_icon = Some(CardIcon {
            background: Some(assets::card_icon(CardIconType::Mana)),
            text: Some(card.data.stored_mana.to_string()),
            background_scale: assets::background_scale(CardIconType::Mana),
        });
    }

    if revealed {
        icons.top_left_icon = queries::mana_cost(game, card.id).map(|mana| CardIcon {
            background: Some(assets::card_icon(CardIconType::Mana)),
            text: Some(mana.to_string()),
            background_scale: assets::background_scale(CardIconType::Mana),
        });
        icons.bottom_left_icon = definition.config.stats.shield.map(|_| CardIcon {
            background: Some(assets::card_icon(CardIconType::Shield)),
            text: Some(queries::shield(game, card.id).to_string()),
            background_scale: assets::background_scale(CardIconType::Shield),
        });
        icons.bottom_right_icon = definition
            .config
            .stats
            .base_attack
            .map(|_| CardIcon {
                background: Some(assets::card_icon(CardIconType::Attack)),
                text: Some(queries::attack(game, card.id).to_string()),
                background_scale: assets::background_scale(CardIconType::Attack),
            })
            .or_else(|| {
                definition.config.stats.health.map(|_| CardIcon {
                    background: Some(assets::card_icon(CardIconType::Health)),
                    text: Some(queries::health(game, card.id).to_string()),
                    background_scale: assets::background_scale(CardIconType::Health),
                })
            });
    }

    icons
}

/// Builds a [RevealedCardView], displaying a card for a user who can currently
/// see this card
fn revealed_card_view(
    game: &GameState,
    card: &CardState,
    definition: &CardDefinition,
    user_side: Side,
) -> RevealedCardView {
    RevealedCardView {
        card_frame: Some(assets::card_frame(definition.school)),
        title_background: Some(assets::title_background(definition.config.faction)),
        jewel: Some(assets::jewel(definition.rarity)),
        image: Some(sprite(&definition.image)),
        title: Some(CardTitle { text: definition.name.displayed_name() }),
        rules_text: Some(rules_text::build(game, card, definition)),
        targeting: Some(card_targeting(game, card, user_side)),
        on_release_position: Some(release_position(definition)),
        supplemental_info: Some(rules_text::build_supplemental_info(game, card, None)),
    }
}

/// Calculates non-standard game object positions
fn position_overrides(game: &GameState, user_side: Side) -> BTreeMap<Id, ObjectPosition> {
    match &game.data.phase {
        GamePhase::ResolveMulligans(mulligans) => {
            opening_hand_position_overrides(game, user_side, mulligans)
        }
        GamePhase::Play => game.data.raid.as_ref().map_or_else(BTreeMap::new, |raid| {
            if raid.phase == RaidPhase::Access {
                raid_access_position_overrides(game, user_side, raid)
            } else {
                raid_position_overrides(game, user_side, raid)
            }
        }),
        GamePhase::GameOver(_) => BTreeMap::new(),
    }
}

/// Positions for cards during the opening hand mulligan decision
fn opening_hand_position_overrides(
    game: &GameState,
    user_side: Side,
    mulligans: &MulliganData,
) -> BTreeMap<Id, ObjectPosition> {
    match mulligans.decision(user_side) {
        None => game
            .hand(user_side)
            .map(|card| {
                (
                    Id::CardId(adapters::adapt_card_id(card.id)),
                    ObjectPosition {
                        sorting_key: card.sorting_key,
                        position: Some(Position::Browser(ObjectPositionBrowser {})),
                    },
                )
            })
            .collect(),
        Some(_) => BTreeMap::new(),
    }
}

/// Positions for game objects during a raid.
fn raid_position_overrides(
    game: &GameState,
    user_side: Side,
    raid: &RaidData,
) -> BTreeMap<Id, ObjectPosition> {
    let mut result = Vec::new();

    match raid.target {
        RoomId::Vault => {
            result.push(Id::Deck(adapters::to_player_name(Side::Overlord, user_side).into()));
        }
        RoomId::Sanctum => {
            result.push(Id::Identity(adapters::to_player_name(Side::Overlord, user_side).into()));
        }
        RoomId::Crypts => {
            result
                .push(Id::DiscardPile(adapters::to_player_name(Side::Overlord, user_side).into()));
        }
        _ => {}
    }

    result.extend(
        game.occupants(raid.target).map(|card| Id::CardId(adapters::adapt_card_id(card.id))),
    );

    let defenders = game.defender_list(raid.target);
    let included = match raid.phase {
        RaidPhase::Activation => &defenders,
        RaidPhase::Encounter(i) => &defenders[..=i],
        RaidPhase::Continue(i) => &defenders[..=i],
        RaidPhase::Access => &[],
    };
    result.extend(included.iter().map(|card| Id::CardId(adapters::adapt_card_id(card.id))));

    result.push(Id::Identity(adapters::to_player_name(Side::Champion, user_side).into()));

    result
        .iter()
        .enumerate()
        .map(|(i, id)| {
            (
                *id,
                ObjectPosition {
                    sorting_key: i as u32,
                    position: Some(Position::Raid(ObjectPositionRaid {})),
                },
            )
        })
        .collect()
}

/// Calculates positions for game objects during the access phase of a raid
fn raid_access_position_overrides(
    game: &GameState,
    _user_side: Side,
    raid: &RaidData,
) -> BTreeMap<Id, ObjectPosition> {
    fn create_position(i: usize, card_id: CardId) -> (Id, ObjectPosition) {
        (
            Id::CardId(adapters::adapt_card_id(card_id)),
            ObjectPosition {
                sorting_key: i as u32,
                position: Some(Position::Browser(ObjectPositionBrowser {})),
            },
        )
    }

    match raid.target {
        RoomId::Sanctum => game
            .hand(Side::Overlord)
            .enumerate()
            .map(|(i, card)| create_position(i, card.id))
            .collect(),
        RoomId::Crypts => game
            .discard_pile(Side::Overlord)
            .enumerate()
            .map(|(i, card)| create_position(i, card.id))
            .collect(),
        _ => raid
            .accessed
            .iter()
            .enumerate()
            .map(|(i, card_id)| create_position(i, *card_id))
            .collect(),
    }
}

/// Converts a card's position into a rendered [ObjectPosition]. Returns None if
/// this [CardPosition] has no equivalent object position, e.g. if the card is
/// currently shuffled into the deck.
pub fn adapt_position(card: &CardState, user_side: Side) -> Option<ObjectPosition> {
    let result = match card.position() {
        CardPosition::Room(room_id, location) => Some(Position::Room(ObjectPositionRoom {
            room_id: adapters::adapt_room_id(room_id).into(),
            room_location: match location {
                RoomLocation::Defender => ClientRoomLocation::Front,
                RoomLocation::Occupant => ClientRoomLocation::Back,
            }
            .into(),
        })),
        CardPosition::ArenaItem(location) => Some(Position::Item(ObjectPositionItem {
            item_location: match location {
                ItemLocation::Weapons => ClientItemLocation::Left,
                ItemLocation::Artifacts => ClientItemLocation::Right,
            }
            .into(),
        })),
        CardPosition::Hand(side) => Some(Position::Hand(ObjectPositionHand {
            owner: adapters::to_player_name(side, user_side).into(),
        })),
        CardPosition::DeckTop(side) => Some(Position::Deck(ObjectPositionDeck {
            owner: adapters::to_player_name(side, user_side).into(),
        })),
        CardPosition::DiscardPile(side) => Some(Position::DiscardPile(ObjectPositionDiscardPile {
            owner: adapters::to_player_name(side, user_side).into(),
        })),
        CardPosition::Scored(side) | CardPosition::Identity(side) => {
            Some(Position::Identity(ObjectPositionIdentity {
                owner: adapters::to_player_name(side, user_side).into(),
            }))
        }
        CardPosition::DeckUnknown(_side) => None,
    };

    result.map(|p| ObjectPosition { sorting_key: card.sorting_key, position: Some(p) })
}

/// Builds a description of the standard [CardTargeting] behavior of a card
fn card_targeting(game: &GameState, card: &CardState, user_side: Side) -> CardTargeting {
    let valid_rooms = || {
        RoomId::into_enum_iter()
            .filter(|room_id| {
                flags::can_take_play_card_action(
                    game,
                    user_side,
                    card.id,
                    CardTarget::Room(*room_id),
                )
            })
            .map(|room_id| adapters::adapt_room_id(room_id).into())
            .collect()
    };

    if let Some(custom_targeting) = &rules::get(card.name).config.custom_targeting {
        return match custom_targeting {
            CustomTargeting::TargetRoom(_) => CardTargeting {
                targeting: Some(Targeting::ArrowTargetRoom(ArrowTargetRoom {
                    valid_rooms: valid_rooms(),
                    arrow: TargetingArrow::Red.into(),
                })),
            },
        };
    }

    CardTargeting {
        targeting: match rules::get(card.name).card_type {
            CardType::Sorcery
            | CardType::Spell
            | CardType::Weapon
            | CardType::Artifact
            | CardType::Identity => Some(Targeting::NoTargeting(NoTargeting {
                can_play: flags::can_take_play_card_action(
                    game,
                    user_side,
                    card.id,
                    CardTarget::None,
                ),
            })),
            CardType::Minion | CardType::Project | CardType::Scheme => {
                Some(Targeting::PlayInRoom(PlayInRoom { valid_rooms: valid_rooms() }))
            }
        },
    }
}

/// Constructs the position to which a card should be moved once it is played
fn release_position(definition: &CardDefinition) -> ObjectPosition {
    ObjectPosition {
        sorting_key: u32::MAX,
        position: Some(match definition.card_type {
            CardType::Sorcery | CardType::Spell | CardType::Identity => {
                Position::Staging(ObjectPositionStaging {})
            }
            CardType::Weapon => Position::Item(ObjectPositionItem {
                item_location: ClientItemLocation::Left.into(),
            }),
            CardType::Artifact => Position::Item(ObjectPositionItem {
                item_location: ClientItemLocation::Right.into(),
            }),
            CardType::Minion => Position::Room(ObjectPositionRoom {
                room_id: RoomIdentifier::Unspecified.into(),
                room_location: ClientRoomLocation::Front.into(),
            }),
            CardType::Project | CardType::Scheme => Position::Room(ObjectPositionRoom {
                room_id: RoomIdentifier::Unspecified.into(),
                room_location: ClientRoomLocation::Back.into(),
            }),
        }),
    }
}

/// Turns a [Sprite] into its protobuf equivalent
fn sprite(sprite: &Sprite) -> SpriteAddress {
    SpriteAddress { address: sprite.address.clone() }
}
