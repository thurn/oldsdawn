#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlexColor {
    ///* Red color component, specified in the range 0.0 to 1.0 inclusive.
    #[prost(float, tag = "1")]
    pub red: f32,
    ///* Green color component, specified in the range 0.0 to 1.0 inclusive.
    #[prost(float, tag = "2")]
    pub green: f32,
    ///* Blue color component, specified in the range 0.0 to 1.0 inclusive.
    #[prost(float, tag = "3")]
    pub blue: f32,
    ///* Alpha color component, specified in the range 0.0 (transparent) to 1.0 (opaque) inclusive.
    #[prost(float, tag = "4")]
    pub alpha: f32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SpriteAddress {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FontAddress {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ProjectileAddress {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EffectAddress {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct AudioClipAddress {
    #[prost(string, tag = "1")]
    pub address: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlexVector2 {
    #[prost(float, tag = "1")]
    pub x: f32,
    #[prost(float, tag = "2")]
    pub y: f32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlexVector3 {
    #[prost(float, tag = "1")]
    pub x: f32,
    #[prost(float, tag = "2")]
    pub y: f32,
    #[prost(float, tag = "3")]
    pub z: f32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Dimension {
    #[prost(enumeration = "DimensionUnit", tag = "1")]
    pub unit: i32,
    #[prost(float, tag = "2")]
    pub value: f32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DimensionGroup {
    #[prost(message, optional, tag = "1")]
    pub top: ::core::option::Option<Dimension>,
    #[prost(message, optional, tag = "2")]
    pub right: ::core::option::Option<Dimension>,
    #[prost(message, optional, tag = "3")]
    pub bottom: ::core::option::Option<Dimension>,
    #[prost(message, optional, tag = "4")]
    pub left: ::core::option::Option<Dimension>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BorderWidth {
    #[prost(float, tag = "1")]
    pub top: f32,
    #[prost(float, tag = "2")]
    pub right: f32,
    #[prost(float, tag = "3")]
    pub bottom: f32,
    #[prost(float, tag = "4")]
    pub left: f32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BorderColor {
    #[prost(message, optional, tag = "1")]
    pub top: ::core::option::Option<FlexColor>,
    #[prost(message, optional, tag = "2")]
    pub right: ::core::option::Option<FlexColor>,
    #[prost(message, optional, tag = "3")]
    pub bottom: ::core::option::Option<FlexColor>,
    #[prost(message, optional, tag = "4")]
    pub left: ::core::option::Option<FlexColor>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct BorderRadius {
    #[prost(message, optional, tag = "1")]
    pub top_left: ::core::option::Option<Dimension>,
    #[prost(message, optional, tag = "2")]
    pub top_right: ::core::option::Option<Dimension>,
    #[prost(message, optional, tag = "3")]
    pub bottom_right: ::core::option::Option<Dimension>,
    #[prost(message, optional, tag = "4")]
    pub bottom_left: ::core::option::Option<Dimension>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlexRotate {
    #[prost(float, tag = "1")]
    pub degrees: f32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlexTranslate {
    #[prost(message, optional, tag = "1")]
    pub x: ::core::option::Option<Dimension>,
    #[prost(message, optional, tag = "2")]
    pub y: ::core::option::Option<Dimension>,
    #[prost(float, tag = "3")]
    pub z: f32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlexScale {
    #[prost(message, optional, tag = "1")]
    pub amount: ::core::option::Option<FlexVector3>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TextShadow {
    #[prost(message, optional, tag = "1")]
    pub offset: ::core::option::Option<FlexVector2>,
    #[prost(float, tag = "2")]
    pub blur_radius: f32,
    #[prost(message, optional, tag = "3")]
    pub color: ::core::option::Option<FlexColor>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct TimeValue {
    #[prost(int32, tag = "1")]
    pub milliseconds: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ImageSlice {
    #[prost(int32, tag = "1")]
    pub top: i32,
    #[prost(int32, tag = "2")]
    pub right: i32,
    #[prost(int32, tag = "3")]
    pub bottom: i32,
    #[prost(int32, tag = "4")]
    pub left: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FlexStyle {
    #[prost(enumeration = "FlexAlign", tag = "1")]
    pub align_content: i32,
    #[prost(enumeration = "FlexAlign", tag = "2")]
    pub align_items: i32,
    #[prost(enumeration = "FlexAlign", tag = "3")]
    pub align_self: i32,
    #[prost(message, optional, tag = "4")]
    pub background_color: ::core::option::Option<FlexColor>,
    #[prost(message, optional, tag = "5")]
    pub background_image: ::core::option::Option<SpriteAddress>,
    #[prost(message, optional, tag = "6")]
    pub border_color: ::core::option::Option<BorderColor>,
    #[prost(message, optional, tag = "7")]
    pub border_radius: ::core::option::Option<BorderRadius>,
    #[prost(message, optional, tag = "8")]
    pub border_width: ::core::option::Option<BorderWidth>,
    #[prost(message, optional, tag = "9")]
    pub inset: ::core::option::Option<DimensionGroup>,
    #[prost(message, optional, tag = "10")]
    pub color: ::core::option::Option<FlexColor>,
    #[prost(enumeration = "FlexDisplayStyle", tag = "11")]
    pub display: i32,
    #[prost(message, optional, tag = "12")]
    pub flex_basis: ::core::option::Option<Dimension>,
    #[prost(enumeration = "FlexDirection", tag = "13")]
    pub flex_direction: i32,
    #[prost(message, optional, tag = "14")]
    pub flex_grow: ::core::option::Option<f32>,
    #[prost(message, optional, tag = "15")]
    pub flex_shrink: ::core::option::Option<f32>,
    #[prost(enumeration = "FlexWrap", tag = "16")]
    pub wrap: i32,
    #[prost(message, optional, tag = "17")]
    pub font_size: ::core::option::Option<Dimension>,
    #[prost(message, optional, tag = "18")]
    pub height: ::core::option::Option<Dimension>,
    #[prost(enumeration = "FlexJustify", tag = "19")]
    pub justify_content: i32,
    #[prost(message, optional, tag = "20")]
    pub letter_spacing: ::core::option::Option<Dimension>,
    #[prost(message, optional, tag = "21")]
    pub margin: ::core::option::Option<DimensionGroup>,
    #[prost(message, optional, tag = "22")]
    pub max_height: ::core::option::Option<Dimension>,
    #[prost(message, optional, tag = "23")]
    pub max_width: ::core::option::Option<Dimension>,
    #[prost(message, optional, tag = "24")]
    pub min_height: ::core::option::Option<Dimension>,
    #[prost(message, optional, tag = "25")]
    pub min_width: ::core::option::Option<Dimension>,
    #[prost(message, optional, tag = "26")]
    pub opacity: ::core::option::Option<f32>,
    #[prost(enumeration = "FlexOverflow", tag = "27")]
    pub overflow: i32,
    #[prost(message, optional, tag = "28")]
    pub padding: ::core::option::Option<DimensionGroup>,
    #[prost(enumeration = "FlexPosition", tag = "29")]
    pub position: i32,
    #[prost(message, optional, tag = "30")]
    pub rotate: ::core::option::Option<FlexRotate>,
    #[prost(message, optional, tag = "31")]
    pub scale: ::core::option::Option<FlexScale>,
    #[prost(enumeration = "TextOverflow", tag = "32")]
    pub text_overflow: i32,
    #[prost(message, optional, tag = "33")]
    pub text_shadow: ::core::option::Option<TextShadow>,
    #[prost(message, optional, tag = "34")]
    pub transform_origin: ::core::option::Option<FlexTranslate>,
    #[prost(message, repeated, tag = "35")]
    pub transition_delays: ::prost::alloc::vec::Vec<TimeValue>,
    #[prost(message, repeated, tag = "36")]
    pub transition_durations: ::prost::alloc::vec::Vec<TimeValue>,
    #[prost(string, repeated, tag = "37")]
    pub transition_properties: ::prost::alloc::vec::Vec<::prost::alloc::string::String>,
    #[prost(enumeration = "EasingMode", repeated, tag = "38")]
    pub transition_easing_modes: ::prost::alloc::vec::Vec<i32>,
    #[prost(message, optional, tag = "39")]
    pub translate: ::core::option::Option<FlexTranslate>,
    #[prost(message, optional, tag = "40")]
    pub background_image_tint_color: ::core::option::Option<FlexColor>,
    #[prost(enumeration = "ImageScaleMode", tag = "41")]
    pub background_image_scale_mode: i32,
    #[prost(message, optional, tag = "42")]
    pub font: ::core::option::Option<FontAddress>,
    #[prost(enumeration = "FontStyle", tag = "43")]
    pub font_style: i32,
    #[prost(enumeration = "OverflowClipBox", tag = "44")]
    pub overflow_clip_box: i32,
    #[prost(message, optional, tag = "45")]
    pub paragraph_spacing: ::core::option::Option<Dimension>,
    #[prost(message, optional, tag = "46")]
    pub image_slice: ::core::option::Option<ImageSlice>,
    #[prost(enumeration = "TextAlign", tag = "47")]
    pub text_align: i32,
    #[prost(message, optional, tag = "48")]
    pub text_outline_color: ::core::option::Option<FlexColor>,
    #[prost(message, optional, tag = "49")]
    pub text_outline_width: ::core::option::Option<f32>,
    #[prost(enumeration = "TextOverflowPosition", tag = "50")]
    pub text_overflow_position: i32,
    #[prost(enumeration = "FlexVisibility", tag = "51")]
    pub visibility: i32,
    #[prost(enumeration = "WhiteSpace", tag = "52")]
    pub white_space: i32,
    #[prost(message, optional, tag = "53")]
    pub width: ::core::option::Option<Dimension>,
    #[prost(message, optional, tag = "54")]
    pub word_spacing: ::core::option::Option<Dimension>,
    ///*
    /// Overwrites both 'width' and 'height' by multiplying the dimensions of
    /// the provided 'background_image' (in units of raw pixels) by this
    /// constant.
    #[prost(message, optional, tag = "55")]
    pub background_image_scale_multiplier: ::core::option::Option<f32>,
    ///*
    /// Calculates the aspect ratio of the provided 'background_image' and uses
    /// it to set dimensions. If 'width' is set, the 'height' will be set based
    /// on the aspect ratio and vice versa. Does not support percentage values.
    #[prost(bool, tag = "56")]
    pub fixed_background_image_aspect_ratio: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Flexbox {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Text {
    #[prost(string, tag = "1")]
    pub label: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EventHandlers {
    #[prost(message, optional, tag = "1")]
    pub on_click: ::core::option::Option<GameAction>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct NodeType {
    #[prost(oneof = "node_type::Type", tags = "1")]
    pub r#type: ::core::option::Option<node_type::Type>,
}
/// Nested message and enum types in `NodeType`.
pub mod node_type {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Type {
        #[prost(message, tag = "1")]
        Text(super::Text),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Node {
    ///*
    /// Used to identify this node in the hierarchy, should be unique among
    /// sibilings. If not provided, index will be used instead.
    #[prost(string, tag = "1")]
    pub id: ::prost::alloc::string::String,
    ///*
    /// Used to identify this node in debugging tools
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "3")]
    pub node_type: ::core::option::Option<NodeType>,
    #[prost(message, repeated, tag = "4")]
    pub children: ::prost::alloc::vec::Vec<Node>,
    #[prost(message, optional, tag = "5")]
    pub event_handlers: ::core::option::Option<EventHandlers>,
    #[prost(message, optional, tag = "6")]
    pub style: ::core::option::Option<FlexStyle>,
    #[prost(message, optional, tag = "7")]
    pub hover_style: ::core::option::Option<FlexStyle>,
    #[prost(message, optional, tag = "8")]
    pub pressed_style: ::core::option::Option<FlexStyle>,
}
// ============================================================================
// Game Primitives
// ============================================================================

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GameId {
    #[prost(string, tag = "1")]
    pub value: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CardId {
    #[prost(int32, tag = "1")]
    pub value: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GameObjectId {
    #[prost(oneof = "game_object_id::Id", tags = "1, 2, 3, 4, 5")]
    pub id: ::core::option::Option<game_object_id::Id>,
}
/// Nested message and enum types in `GameObjectId`.
pub mod game_object_id {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Id {
        #[prost(message, tag = "1")]
        CardId(super::CardId),
        #[prost(enumeration = "super::PlayerName", tag = "2")]
        Identity(i32),
        #[prost(enumeration = "super::PlayerName", tag = "3")]
        Deck(i32),
        #[prost(enumeration = "super::PlayerName", tag = "4")]
        Hand(i32),
        #[prost(enumeration = "super::PlayerName", tag = "5")]
        DiscardPile(i32),
    }
}
// ============================================================================
// Game View
// ============================================================================

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CardIcon {
    #[prost(message, optional, tag = "1")]
    pub background: ::core::option::Option<SpriteAddress>,
    #[prost(string, tag = "2")]
    pub text: ::prost::alloc::string::String,
    #[prost(float, tag = "3")]
    pub background_scale: f32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CardIcons {
    #[prost(message, optional, tag = "1")]
    pub top_left_icon: ::core::option::Option<CardIcon>,
    #[prost(message, optional, tag = "2")]
    pub top_right_icon: ::core::option::Option<CardIcon>,
    #[prost(message, optional, tag = "3")]
    pub bottom_right_icon: ::core::option::Option<CardIcon>,
    #[prost(message, optional, tag = "4")]
    pub bottom_left_icon: ::core::option::Option<CardIcon>,
    #[prost(message, optional, tag = "5")]
    pub arena_icon: ::core::option::Option<CardIcon>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CardTitle {
    #[prost(string, tag = "1")]
    pub text: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RulesText {
    #[prost(string, tag = "1")]
    pub text: ::prost::alloc::string::String,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PickRoom {
    #[prost(enumeration = "RoomId", repeated, tag = "2")]
    pub valid_rooms: ::prost::alloc::vec::Vec<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CardTargeting {
    #[prost(oneof = "card_targeting::Targeting", tags = "1")]
    pub targeting: ::core::option::Option<card_targeting::Targeting>,
}
/// Nested message and enum types in `CardTargeting`.
pub mod card_targeting {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Targeting {
        ///* Pick one of these valid rooms.
        #[prost(message, tag = "1")]
        PickRoom(super::PickRoom),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CardCost {
    #[prost(int32, tag = "1")]
    pub mana_cost: i32,
    #[prost(int32, tag = "2")]
    pub action_cost: i32,
    #[prost(bool, tag = "3")]
    pub can_play: bool,
    #[prost(enumeration = "CanPlayAlgorithm", tag = "4")]
    pub can_play_algorithm: i32,
    #[prost(enumeration = "SpendCostAlgorithm", tag = "5")]
    pub spend_cost_algorithm: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectPositionOffscreen {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectPositionRoom {
    #[prost(enumeration = "RoomId", tag = "1")]
    pub room_id: i32,
    #[prost(enumeration = "RoomLocation", tag = "2")]
    pub room_location: i32,
    #[prost(message, optional, tag = "3")]
    pub index: ::core::option::Option<i32>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectPositionItem {
    #[prost(enumeration = "ItemLocation", tag = "1")]
    pub item_location: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectPositionStaging {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectPositionHand {
    #[prost(enumeration = "PlayerName", tag = "1")]
    pub owner: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectPositionDeck {
    #[prost(enumeration = "PlayerName", tag = "1")]
    pub owner: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectPositionDeckContainer {
    #[prost(enumeration = "PlayerName", tag = "1")]
    pub owner: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectPositionDiscardPile {
    #[prost(enumeration = "PlayerName", tag = "1")]
    pub owner: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectPositionDiscardPileContainer {
    #[prost(enumeration = "PlayerName", tag = "1")]
    pub owner: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectPositionScored {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectPositionRaid {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectPositionBrowser {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectPositionIdentity {
    #[prost(enumeration = "PlayerName", tag = "1")]
    pub owner: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectPositionIdentityContainer {
    #[prost(enumeration = "PlayerName", tag = "1")]
    pub owner: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectPositionRewardChest {}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ObjectPosition {
    #[prost(
        oneof = "object_position::Position",
        tags = "1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14"
    )]
    pub position: ::core::option::Option<object_position::Position>,
}
/// Nested message and enum types in `ObjectPosition`.
pub mod object_position {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Position {
        #[prost(message, tag = "1")]
        Offscreen(super::ObjectPositionOffscreen),
        #[prost(message, tag = "2")]
        Room(super::ObjectPositionRoom),
        #[prost(message, tag = "3")]
        Item(super::ObjectPositionItem),
        #[prost(message, tag = "4")]
        Staging(super::ObjectPositionStaging),
        #[prost(message, tag = "5")]
        Hand(super::ObjectPositionHand),
        #[prost(message, tag = "6")]
        Deck(super::ObjectPositionDeck),
        #[prost(message, tag = "7")]
        DeckContainer(super::ObjectPositionDeckContainer),
        #[prost(message, tag = "8")]
        DiscardPile(super::ObjectPositionDiscardPile),
        #[prost(message, tag = "9")]
        DiscardPileContainer(super::ObjectPositionDiscardPileContainer),
        #[prost(message, tag = "10")]
        Scored(super::ObjectPositionScored),
        #[prost(message, tag = "11")]
        Raid(super::ObjectPositionRaid),
        #[prost(message, tag = "12")]
        Browser(super::ObjectPositionBrowser),
        #[prost(message, tag = "13")]
        Identity(super::ObjectPositionIdentity),
        #[prost(message, tag = "14")]
        IdentityContainer(super::ObjectPositionIdentityContainer),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RevealedCardView {
    #[prost(message, optional, tag = "1")]
    pub card_frame: ::core::option::Option<SpriteAddress>,
    #[prost(message, optional, tag = "2")]
    pub title_background: ::core::option::Option<SpriteAddress>,
    #[prost(message, optional, tag = "3")]
    pub jewel: ::core::option::Option<SpriteAddress>,
    #[prost(message, optional, tag = "4")]
    pub image_background: ::core::option::Option<SpriteAddress>,
    #[prost(message, optional, tag = "5")]
    pub image: ::core::option::Option<SpriteAddress>,
    #[prost(message, optional, tag = "6")]
    pub title: ::core::option::Option<CardTitle>,
    #[prost(message, optional, tag = "7")]
    pub rules_text: ::core::option::Option<RulesText>,
    ///*
    /// True if this card should be displayed as visible to the opponent when in the arena.
    #[prost(bool, tag = "8")]
    pub revealed_in_arena: bool,
    ///*
    /// Custom targeting behavior for a card. If unspecified, no targeting UI
    /// is shown.
    #[prost(message, optional, tag = "9")]
    pub targeting: ::core::option::Option<CardTargeting>,
    ///*
    /// Where to move a played card. Information from 'targeting' will be
    /// incorporated to fill this in, e.g. if a room is targeted and
    /// ObjectPositionRoom is selected here with no RoomId, the targeted room
    /// is used.
    #[prost(message, optional, tag = "10")]
    pub on_release_position: ::core::option::Option<ObjectPosition>,
    ///* Information needed to determine whether a card can be played.
    #[prost(message, optional, tag = "11")]
    pub cost: ::core::option::Option<CardCost>,
    ///* Additional interface element rendered to the side of the card during an info zoom.
    #[prost(message, optional, tag = "12")]
    pub supplemental_info: ::core::option::Option<Node>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CardView {
    #[prost(message, optional, tag = "1")]
    pub card_id: ::core::option::Option<CardId>,
    #[prost(message, optional, tag = "2")]
    pub card_icons: ::core::option::Option<CardIcons>,
    #[prost(message, optional, tag = "3")]
    pub arena_frame: ::core::option::Option<SpriteAddress>,
    ///* Used to e.g. determine which card back to display for this card.
    #[prost(enumeration = "PlayerName", tag = "4")]
    pub owning_player: i32,
    #[prost(message, optional, tag = "5")]
    pub revealed_card: ::core::option::Option<RevealedCardView>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct HandView {
    #[prost(message, repeated, tag = "1")]
    pub cards: ::prost::alloc::vec::Vec<CardView>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DiscardPileView {
    #[prost(message, repeated, tag = "1")]
    pub cards: ::prost::alloc::vec::Vec<CardView>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlayerInfo {
    #[prost(string, tag = "1")]
    pub name: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "2")]
    pub portrait: ::core::option::Option<SpriteAddress>,
    #[prost(message, optional, tag = "3")]
    pub portrait_frame: ::core::option::Option<SpriteAddress>,
    ///* Card back asset to use for this player's cards.
    #[prost(message, optional, tag = "4")]
    pub card_back: ::core::option::Option<SpriteAddress>,
    ///* Describes the player's unique powers.
    #[prost(message, optional, tag = "5")]
    pub identity_card: ::core::option::Option<CardView>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ManaView {
    #[prost(int32, tag = "1")]
    pub amount: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ScoreView {
    #[prost(int32, tag = "1")]
    pub score: i32,
    #[prost(message, repeated, tag = "2")]
    pub scored_cards: ::prost::alloc::vec::Vec<CardView>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RoomView {
    #[prost(enumeration = "RoomId", tag = "1")]
    pub room_id: i32,
    #[prost(message, repeated, tag = "2")]
    pub back_cards: ::prost::alloc::vec::Vec<CardView>,
    #[prost(message, repeated, tag = "3")]
    pub front_cards: ::prost::alloc::vec::Vec<CardView>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ArenaView {
    ///*
    /// If true, render rooms at the bottom of the screen, if false, render items
    /// at the bottom.
    #[prost(message, optional, tag = "1")]
    pub rooms_at_bottom: ::core::option::Option<bool>,
    ///*
    /// Controls the drag action taken for the player's identity card.
    #[prost(enumeration = "IdentityAction", tag = "2")]
    pub identity_action: i32,
    #[prost(message, repeated, tag = "3")]
    pub rooms: ::prost::alloc::vec::Vec<RoomView>,
    #[prost(message, repeated, tag = "4")]
    pub left_items: ::prost::alloc::vec::Vec<CardView>,
    #[prost(message, repeated, tag = "5")]
    pub right_items: ::prost::alloc::vec::Vec<CardView>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ActionTrackerView {
    #[prost(int32, tag = "1")]
    pub available_action_count: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DeckView {
    #[prost(message, repeated, tag = "1")]
    pub top_cards: ::prost::alloc::vec::Vec<CardView>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlayerView {
    #[prost(message, optional, tag = "1")]
    pub player_info: ::core::option::Option<PlayerInfo>,
    #[prost(message, optional, tag = "2")]
    pub score: ::core::option::Option<ScoreView>,
    #[prost(message, optional, tag = "3")]
    pub hand: ::core::option::Option<HandView>,
    #[prost(message, optional, tag = "4")]
    pub mana: ::core::option::Option<ManaView>,
    #[prost(message, optional, tag = "5")]
    pub discard_pile: ::core::option::Option<DiscardPileView>,
    #[prost(message, optional, tag = "6")]
    pub action_tracker: ::core::option::Option<ActionTrackerView>,
    #[prost(message, optional, tag = "7")]
    pub deck: ::core::option::Option<DeckView>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GameView {
    #[prost(message, optional, tag = "1")]
    pub game_id: ::core::option::Option<GameId>,
    #[prost(message, optional, tag = "2")]
    pub user: ::core::option::Option<PlayerView>,
    #[prost(message, optional, tag = "3")]
    pub opponent: ::core::option::Option<PlayerView>,
    #[prost(message, optional, tag = "4")]
    pub arena: ::core::option::Option<ArenaView>,
    ///* The player who is currently able to act.
    #[prost(enumeration = "PlayerName", tag = "5")]
    pub current_priority: i32,
}
// ============================================================================
// Actions
// ============================================================================

///*
/// Start a game. If game_id is provided, connects to an ongoing game.
/// Otherwise creates a new game.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ConnectAction {
    #[prost(message, optional, tag = "1")]
    pub game_id: ::core::option::Option<GameId>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct StandardAction {
    ///* Opaque payload to send to the server.
    #[prost(message, optional, tag = "1")]
    pub payload: ::core::option::Option<::prost_types::Any>,
    ///* Immediate optimistic mutations to game state for this action.
    #[prost(message, optional, tag = "2")]
    pub update: ::core::option::Option<CommandList>,
}
///*
/// Spend an action to gain 1 mana.
///
/// Optimistic: Mana is added immediately.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GainManaAction {}
///*
/// Spend an action to draw a card.
///
/// Optimistic: Face-down card animates to reveal area.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DrawCardAction {}
///*
/// Spend an action to level up a room.
///
/// Optimistic: Counter is added immediately
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LevelUpRoomAction {
    #[prost(enumeration = "RoomId", tag = "1")]
    pub room_id: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CardTarget {
    #[prost(oneof = "card_target::CardTarget", tags = "1")]
    pub card_target: ::core::option::Option<card_target::CardTarget>,
}
/// Nested message and enum types in `CardTarget`.
pub mod card_target {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum CardTarget {
        #[prost(enumeration = "super::RoomId", tag = "1")]
        RoomId(i32),
    }
}
///*
/// Spend an action to play a card from hand.
///
/// Optimistic:
///   - Mana and action cost is spent immediately, and 'can play' values for
///     other cards in hand are optimistically updated
///   - Other costs like 'sacrifice an artifact' are not optimistic and are
///     handled like choices
///   - Targeted cards select their *first* valid target (cards, rooms, players)
///     optimistically. If additional targets are required, they're not handled
///     optimistically, and this play pattern should possibly be avoided.
///   - Cards that require a choice to be made before resolving do not display
///     the options optimistically, instead they animate to the reveal card area
///   - Item cards which don't require a choice to be made or target simply
///     animate into the play area optimistically
///   - Spell cards animate to the reveal card area and wait for their effects to be
///     applied
///   - Minion and Project cards animate to their selected room optimistically
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlayCardAction {
    #[prost(message, optional, tag = "1")]
    pub card_id: ::core::option::Option<CardId>,
    #[prost(message, optional, tag = "2")]
    pub target: ::core::option::Option<CardTarget>,
}
///*
/// Spend an action to initiate a raid on one of the overlord's rooms
///
/// Optimistic: Raid start animation plays
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InitiateRaidAction {
    #[prost(enumeration = "RoomId", tag = "1")]
    pub room_id: i32,
}
///*
/// Possible game actions taken by the user.
///
/// Actions have an associated 'optimistic' behavior to display while waiting
/// for a server response. The client should not send multiple actions at the
/// same time -- interaction should be disabled while an action is pending.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GameAction {
    #[prost(oneof = "game_action::Action", tags = "1, 2, 3, 4, 5, 6, 7")]
    pub action: ::core::option::Option<game_action::Action>,
}
/// Nested message and enum types in `GameAction`.
pub mod game_action {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Action {
        #[prost(message, tag = "1")]
        StandardAction(super::StandardAction),
        #[prost(message, tag = "2")]
        Connect(super::ConnectAction),
        #[prost(message, tag = "3")]
        GainMana(super::GainManaAction),
        #[prost(message, tag = "4")]
        DrawCard(super::DrawCardAction),
        #[prost(message, tag = "5")]
        PlayCard(super::PlayCardAction),
        #[prost(message, tag = "6")]
        LevelUpRoom(super::LevelUpRoomAction),
        #[prost(message, tag = "7")]
        InitiateRaid(super::InitiateRaidAction),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GameRequest {
    #[prost(message, optional, tag = "1")]
    pub action: ::core::option::Option<GameAction>,
    ///* Current game_id, if a game is currently ongoing.
    #[prost(message, optional, tag = "2")]
    pub game_id: ::core::option::Option<GameId>,
}
// ============================================================================
// Commands
// ============================================================================

#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DebugLogCommand {
    #[prost(string, tag = "1")]
    pub message: ::prost::alloc::string::String,
}
///*
/// Run a series of command lists simultaneously. Warning: applying multiple commands
/// to the same game object will have unpredictable results.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RunInParallelCommand {
    #[prost(message, repeated, tag = "1")]
    pub commands: ::prost::alloc::vec::Vec<CommandList>,
}
///* Wait before executing the next command in sequence.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DelayCommand {
    #[prost(message, optional, tag = "1")]
    pub duration: ::core::option::Option<TimeValue>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InterfacePositionFullScreen {
    #[prost(message, optional, tag = "1")]
    pub node: ::core::option::Option<Node>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InterfacePositionMainControls {
    #[prost(message, optional, tag = "1")]
    pub node: ::core::option::Option<Node>,
}
///* Render an interface element attached to a specific card.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CardAnchorNode {
    #[prost(message, optional, tag = "1")]
    pub card_id: ::core::option::Option<CardId>,
    #[prost(message, optional, tag = "2")]
    pub node: ::core::option::Option<Node>,
    #[prost(enumeration = "CardNodeAnchorPosition", tag = "3")]
    pub anchor_position: i32,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InterfacePositionCardAnchors {
    #[prost(message, repeated, tag = "1")]
    pub anchor_nodes: ::prost::alloc::vec::Vec<CardAnchorNode>,
}
///*
/// Updates the content of the user interface to display the provided node,
/// replacing all existing UI elements.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RenderInterfaceCommand {
    #[prost(oneof = "render_interface_command::Position", tags = "1, 2, 3")]
    pub position: ::core::option::Option<render_interface_command::Position>,
}
/// Nested message and enum types in `RenderInterfaceCommand`.
pub mod render_interface_command {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Position {
        #[prost(message, tag = "1")]
        FullScreen(super::InterfacePositionFullScreen),
        #[prost(message, tag = "2")]
        MainControls(super::InterfacePositionMainControls),
        #[prost(message, tag = "3")]
        CardAnchors(super::InterfacePositionCardAnchors),
    }
}
///*
/// Many of the below commands are specific cases of RenderGame. They are
/// differentiated in order to simplify the diffing logic the client needs
/// to perform to detect and animate changes.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct RenderGameCommand {
    #[prost(message, optional, tag = "1")]
    pub game: ::core::option::Option<GameView>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct InitiateRaidCommand {
    #[prost(enumeration = "PlayerName", tag = "1")]
    pub initiator: i32,
    #[prost(enumeration = "RoomId", tag = "2")]
    pub room_id: i32,
}
///*
/// Removes the raid UI overlay. This command does not cause game objects within
/// the raid to change position, 'MoveGameObjectCommand' should be used for that
/// instead.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EndRaidCommand {
    #[prost(enumeration = "PlayerName", tag = "1")]
    pub initiator: i32,
}
///*
/// Animates 'initiator' moving to a room and plays a standard effect.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LevelUpRoomCommand {
    #[prost(enumeration = "PlayerName", tag = "1")]
    pub initiator: i32,
    #[prost(enumeration = "RoomId", tag = "2")]
    pub room_id: i32,
}
///*
/// Makes a new card.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CreateCardCommand {
    #[prost(message, optional, tag = "1")]
    pub card: ::core::option::Option<CardView>,
    #[prost(message, optional, tag = "2")]
    pub position: ::core::option::Option<ObjectPosition>,
    ///* Optionally, an animation to play after creating the card.
    #[prost(enumeration = "CardCreationAnimation", tag = "3")]
    pub animation: i32,
    ///*
    /// Disable animations when creating this card.
    #[prost(bool, tag = "4")]
    pub disable_animation: bool,
}
///*
/// Updates a card. Note that changes to 'on_create_position' are ignored here,
/// use MoveCardCommand instead to reposition cards.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdateCardCommand {
    #[prost(message, optional, tag = "1")]
    pub card: ::core::option::Option<CardView>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MoveGameObjectsCommand {
    #[prost(message, repeated, tag = "1")]
    pub ids: ::prost::alloc::vec::Vec<GameObjectId>,
    #[prost(message, optional, tag = "2")]
    pub position: ::core::option::Option<ObjectPosition>,
    ///*
    /// Position at which to insert. If multiple IDs are specified, they will
    /// be sequentially added at this position
    #[prost(message, optional, tag = "3")]
    pub index: ::core::option::Option<i32>,
    #[prost(bool, tag = "4")]
    pub disable_animation: bool,
}
///* Request to move all GameObjects located at a specific object position to another object position.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct MoveGameObjectsAtPositionCommand {
    #[prost(message, optional, tag = "1")]
    pub source_position: ::core::option::Option<ObjectPosition>,
    #[prost(message, optional, tag = "2")]
    pub target_position: ::core::option::Option<ObjectPosition>,
    #[prost(bool, tag = "3")]
    pub disable_animation: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DestroyCardCommand {
    #[prost(message, optional, tag = "1")]
    pub card_id: ::core::option::Option<CardId>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct UpdatePlayerStateCommand {
    #[prost(enumeration = "PlayerName", tag = "1")]
    pub player_name: i32,
    #[prost(message, optional, tag = "2")]
    pub info: ::core::option::Option<PlayerInfo>,
    #[prost(message, optional, tag = "3")]
    pub score: ::core::option::Option<ScoreView>,
    #[prost(message, optional, tag = "4")]
    pub action_tracker: ::core::option::Option<ActionTrackerView>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlaySoundCommand {
    #[prost(message, optional, tag = "1")]
    pub sound: ::core::option::Option<AudioClipAddress>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetMusicCommand {
    #[prost(enumeration = "MusicState", tag = "1")]
    pub music_state: i32,
}
///*
/// Fire a projectile from one game object at another.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct FireProjectileCommand {
    #[prost(message, optional, tag = "1")]
    pub source_id: ::core::option::Option<GameObjectId>,
    #[prost(message, optional, tag = "2")]
    pub target_id: ::core::option::Option<GameObjectId>,
    ///* Projectile to fire from the 'source_id' card to 'target_id'
    #[prost(message, optional, tag = "3")]
    pub projectile: ::core::option::Option<ProjectileAddress>,
    ///* How long the projectile should take to hit its target.
    #[prost(message, optional, tag = "4")]
    pub travel_duration: ::core::option::Option<TimeValue>,
    #[prost(message, optional, tag = "5")]
    pub fire_sound: ::core::option::Option<AudioClipAddress>,
    #[prost(message, optional, tag = "6")]
    pub impact_sound: ::core::option::Option<AudioClipAddress>,
    ///* Additional effect to display on the target on hit.
    #[prost(message, optional, tag = "7")]
    pub additional_hit: ::core::option::Option<EffectAddress>,
    ///*
    /// Delay before showing the additional hit. If provided, the original
    /// projectile Hit effect will be hidden before showing the new hit effect.
    #[prost(message, optional, tag = "8")]
    pub additional_hit_delay: ::core::option::Option<TimeValue>,
    ///*
    /// During to wait for the project's impact effect before continuing
    #[prost(message, optional, tag = "9")]
    pub wait_duration: ::core::option::Option<TimeValue>,
    ///*
    /// If true, the target will be hidden after being hit during the
    /// 'wait_duration' and before jumping to 'jump_to_position'.
    #[prost(bool, tag = "10")]
    pub hide_on_hit: bool,
    ///*
    /// Position for the target to jump to after being hit.
    #[prost(message, optional, tag = "11")]
    pub jump_to_position: ::core::option::Option<ObjectPosition>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlayEffectPosition {
    #[prost(oneof = "play_effect_position::EffectPosition", tags = "1")]
    pub effect_position: ::core::option::Option<play_effect_position::EffectPosition>,
}
/// Nested message and enum types in `PlayEffectPosition`.
pub mod play_effect_position {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum EffectPosition {
        #[prost(message, tag = "1")]
        GameObject(super::GameObjectId),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlayEffectCommand {
    #[prost(message, optional, tag = "1")]
    pub effect: ::core::option::Option<EffectAddress>,
    #[prost(message, optional, tag = "2")]
    pub position: ::core::option::Option<PlayEffectPosition>,
    #[prost(message, optional, tag = "3")]
    pub scale: ::core::option::Option<f32>,
    ///* How long to wait before continuing.
    #[prost(message, optional, tag = "4")]
    pub duration: ::core::option::Option<TimeValue>,
    #[prost(message, optional, tag = "5")]
    pub sound: ::core::option::Option<AudioClipAddress>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DisplayGameMessageCommand {
    #[prost(enumeration = "GameMessageType", tag = "1")]
    pub message_type: i32,
}
///* Used to hide and show all game UI elements.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct SetGameObjectsEnabledCommand {
    #[prost(bool, tag = "1")]
    pub game_objects_enabled: bool,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct DisplayRewardsCommand {
    #[prost(message, repeated, tag = "1")]
    pub rewards: ::prost::alloc::vec::Vec<CardView>,
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct GameCommand {
    #[prost(
        oneof = "game_command::Command",
        tags = "1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21"
    )]
    pub command: ::core::option::Option<game_command::Command>,
}
/// Nested message and enum types in `GameCommand`.
pub mod game_command {
    #[derive(Clone, PartialEq, ::prost::Oneof)]
    pub enum Command {
        #[prost(message, tag = "1")]
        DebugLog(super::DebugLogCommand),
        #[prost(message, tag = "2")]
        RunInParallel(super::RunInParallelCommand),
        #[prost(message, tag = "3")]
        Delay(super::DelayCommand),
        #[prost(message, tag = "4")]
        RenderInterface(super::RenderInterfaceCommand),
        #[prost(message, tag = "5")]
        RenderGame(super::RenderGameCommand),
        #[prost(message, tag = "6")]
        InitiateRaid(super::InitiateRaidCommand),
        #[prost(message, tag = "7")]
        EndRaid(super::EndRaidCommand),
        #[prost(message, tag = "8")]
        LevelUpRoom(super::LevelUpRoomCommand),
        #[prost(message, tag = "9")]
        CreateCard(super::CreateCardCommand),
        #[prost(message, tag = "10")]
        UpdateCard(super::UpdateCardCommand),
        #[prost(message, tag = "11")]
        MoveGameObjects(super::MoveGameObjectsCommand),
        #[prost(message, tag = "12")]
        MoveObjectsAtPosition(super::MoveGameObjectsAtPositionCommand),
        #[prost(message, tag = "13")]
        DestroyCard(super::DestroyCardCommand),
        #[prost(message, tag = "14")]
        UpdatePlayerState(super::UpdatePlayerStateCommand),
        #[prost(message, tag = "15")]
        PlaySound(super::PlaySoundCommand),
        #[prost(message, tag = "16")]
        SetMusic(super::SetMusicCommand),
        #[prost(message, tag = "17")]
        FireProjectile(super::FireProjectileCommand),
        #[prost(message, tag = "18")]
        PlayEffect(super::PlayEffectCommand),
        #[prost(message, tag = "19")]
        DisplayGameMessage(super::DisplayGameMessageCommand),
        #[prost(message, tag = "20")]
        SetGameObjectsEnabled(super::SetGameObjectsEnabledCommand),
        #[prost(message, tag = "21")]
        DisplayRewards(super::DisplayRewardsCommand),
    }
}
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CommandList {
    #[prost(message, repeated, tag = "1")]
    pub commands: ::prost::alloc::vec::Vec<GameCommand>,
}
// ============================================================================
// Masonry
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum FlexAlign {
    Unspecified = 0,
    Auto = 1,
    FlexStart = 2,
    Center = 3,
    FlexEnd = 4,
    Stretch = 5,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum FlexDisplayStyle {
    Unspecified = 0,
    Flex = 1,
    None = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum FlexDirection {
    Unspecified = 0,
    Column = 1,
    ColumnReverse = 2,
    Row = 3,
    RowReverse = 4,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum FlexWrap {
    Unspecified = 0,
    NoWrap = 1,
    Wrap = 2,
    WrapReverse = 3,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum FlexJustify {
    Unspecified = 0,
    FlexStart = 1,
    Center = 2,
    FlexEnd = 3,
    SpaceBetween = 4,
    SpaceAround = 5,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum FlexOverflow {
    Unspecified = 0,
    Visible = 1,
    Hidden = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum FlexPosition {
    Unspecified = 0,
    Relative = 1,
    Absolute = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum TextOverflow {
    Unspecified = 0,
    Clip = 1,
    Ellipsis = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum EasingMode {
    Unspecified = 0,
    Ease = 1,
    EaseIn = 2,
    EaseOut = 3,
    EaseInOut = 4,
    Linear = 5,
    EaseInSine = 6,
    EaseOutSine = 7,
    EaseInOutSine = 8,
    EaseInCubic = 9,
    EaseOutCubic = 10,
    EaseInOutCubic = 11,
    EaseInCirc = 12,
    EaseOutCirc = 13,
    EaseInOutCirc = 14,
    EaseInElastic = 15,
    EaseOutElastic = 16,
    EaseInOutElastic = 17,
    EaseInBack = 18,
    EaseOutBack = 19,
    EaseInOutBack = 20,
    EaseInBounce = 21,
    EaseOutBounce = 22,
    EaseInOutBounce = 23,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ImageScaleMode {
    Unspecified = 0,
    StretchToFill = 1,
    ScaleAndCrop = 2,
    ScaleToFit = 3,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum FontStyle {
    Unspecified = 0,
    Normal = 1,
    Bold = 2,
    Italic = 3,
    BoldAndItalic = 4,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum OverflowClipBox {
    Unspecified = 0,
    PaddingBox = 1,
    ContentBox = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum TextAlign {
    Unspecified = 0,
    UpperLeft = 1,
    UpperCenter = 2,
    UpperRight = 3,
    MiddleLeft = 4,
    MiddleCenter = 5,
    MiddleRight = 6,
    LowerLeft = 7,
    LowerCenter = 8,
    LowerRight = 9,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum TextOverflowPosition {
    Unspecified = 0,
    End = 1,
    Start = 2,
    Middle = 3,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum FlexVisibility {
    Unspecified = 0,
    Visible = 1,
    Hidden = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum WhiteSpace {
    Unspecified = 0,
    Normal = 1,
    NoWrap = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum DimensionUnit {
    Unspecified = 0,
    ///* Density-independent pixels, which unity also calls "pixels".
    Dip = 1,
    Percentage = 2,
    Vmin = 3,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum PlayerName {
    Unspecified = 0,
    User = 1,
    Opponent = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum RoomId {
    Unspecified = 0,
    Treasury = 1,
    Sanctum = 2,
    Crypts = 3,
    RoomA = 4,
    RoomB = 5,
    RoomC = 6,
    RoomD = 7,
    RoomE = 8,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum TargetingArrow {
    Unspecified = 0,
    Red = 1,
    Blue = 2,
    Green = 3,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CanPlayAlgorithm {
    Unspecified = 0,
    ///*
    /// Always update the 'can_play' state automatically based on available mana
    /// and actions.
    Optimistic = 1,
    ///*
    /// 'Optimistic False' algorithm for cards with an additional cost. Will
    /// transition the 'can_play' state to false based on available mana and
    /// actions, but not to true.
    AdditionalCost = 2,
    ///*
    /// 'Optimistic True' algorithm, for cards which can always be played for
    /// their normal cost, but have other valid play windows as well. Will
    /// transition the 'can_play' state to true based on available mana and
    /// actions, but not to false.
    AdditionalPlay = 3,
    ///*
    /// Do not transition the 'can_play' state automatically
    NoUpdate = 4,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum SpendCostAlgorithm {
    Unspecified = 0,
    ///* Deduct the mana & action cost immediately when this card is played.
    Optimistic = 1,
    ///* Do not modify mana & action values when this card is played.
    NoUpdate = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum RoomLocation {
    Unspecified = 0,
    Back = 1,
    Front = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ItemLocation {
    Unspecified = 0,
    Left = 1,
    Right = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum IdentityAction {
    Unspecified = 0,
    InitiateRaid = 1,
    LevelUpRoom = 2,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CardNodeAnchorPosition {
    Unspecified = 0,
    Bottom = 1,
    Left = 2,
    Right = 3,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum CardCreationAnimation {
    Unspecified = 0,
    UserDeckToStaging = 1,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum MusicState {
    Unspecified = 0,
    Silent = 1,
    Gameplay = 2,
    Raid = 3,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum GameMessageType {
    Unspecified = 0,
    Dawn = 1,
    Dusk = 2,
    Victory = 3,
    Defeat = 4,
}
#[doc = r" Generated server implementations."]
pub mod spelldawn_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    #[doc = "Generated trait containing gRPC methods that should be implemented for use with SpelldawnServer."]
    #[async_trait]
    pub trait Spelldawn: Send + Sync + 'static {
        #[doc = " Our SayHello rpc accepts HelloRequests and returns HelloReplies"]
        async fn perform_action(
            &self,
            request: tonic::Request<super::GameRequest>,
        ) -> Result<tonic::Response<super::CommandList>, tonic::Status>;
    }
    #[derive(Debug)]
    pub struct SpelldawnServer<T: Spelldawn> {
        inner: _Inner<T>,
        accept_compression_encodings: (),
        send_compression_encodings: (),
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Spelldawn> SpelldawnServer<T> {
        pub fn new(inner: T) -> Self {
            let inner = Arc::new(inner);
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
            }
        }
        pub fn with_interceptor<F>(inner: T, interceptor: F) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for SpelldawnServer<T>
    where
        T: Spelldawn,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = Never;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/spelldawn.Spelldawn/PerformAction" => {
                    #[allow(non_camel_case_types)]
                    struct PerformActionSvc<T: Spelldawn>(pub Arc<T>);
                    impl<T: Spelldawn> tonic::server::UnaryService<super::GameRequest> for PerformActionSvc<T> {
                        type Response = super::CommandList;
                        type Future = BoxFuture<tonic::Response<Self::Response>, tonic::Status>;
                        fn call(
                            &mut self,
                            request: tonic::Request<super::GameRequest>,
                        ) -> Self::Future {
                            let inner = self.0.clone();
                            let fut = async move { (*inner).perform_action(request).await };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = PerformActionSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec).apply_compression_config(
                            accept_compression_encodings,
                            send_compression_encodings,
                        );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => Box::pin(async move {
                    Ok(http::Response::builder()
                        .status(200)
                        .header("grpc-status", "12")
                        .header("content-type", "application/grpc")
                        .body(empty_body())
                        .unwrap())
                }),
            }
        }
    }
    impl<T: Spelldawn> Clone for SpelldawnServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
            }
        }
    }
    impl<T: Spelldawn> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Spelldawn> tonic::transport::NamedService for SpelldawnServer<T> {
        const NAME: &'static str = "spelldawn.Spelldawn";
    }
}
