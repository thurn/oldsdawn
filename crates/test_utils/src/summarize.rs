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

use std::fmt::Debug;

use adapters;
use adapters::ServerCardId;
use anyhow::Result;
use protos::spelldawn::card_targeting::Targeting;
use protos::spelldawn::game_command::Command;
use protos::spelldawn::game_object_identifier::Id;
use protos::spelldawn::object_position::Position;
use protos::spelldawn::play_effect_position::EffectPosition;
use protos::spelldawn::{
    node_type, ActionTrackerView, AnchorCorner, ArrowTargetRoom, AudioClipAddress, CardAnchor,
    CardAnchorNode, CardCreationAnimation, CardIcon, CardIcons, CardIdentifier, CardTargeting,
    CardTitle, CardView, CommandList, ConnectToGameCommand, CreateTokenCardCommand, DelayCommand,
    DisplayGameMessageCommand, DisplayRewardsCommand, EffectAddress, FireProjectileCommand,
    GameCommand, GameMessageType, GameObjectIdentifier, GameObjectMove, GameObjectPositions,
    GameView, InterfaceMainControls, InterfacePanel, LoadSceneCommand, ManaView,
    MoveGameObjectsCommand, MusicState, NoTargeting, Node, NodeType, ObjectPosition, PanelAddress,
    PlayEffectCommand, PlayEffectPosition, PlayInRoom, PlaySoundCommand, PlayerInfo, PlayerName,
    PlayerSide, PlayerView, ProjectileAddress, RevealedCardView, RoomIdentifier, RoomVisitType,
    RulesText, RunInParallelCommand, SceneLoadMode, ScoreView, SetGameObjectsEnabledCommand,
    SetMusicCommand, SetPlayerIdentifierCommand, SpriteAddress, TimeValue, TogglePanelCommand,
    UpdateGameViewCommand, UpdatePanelsCommand, VisitRoomCommand,
};
use server::requests::GameResponse;

pub trait Summarize {
    fn summarize(self, summary: &mut Summary);
}

pub struct Summary {
    value: String,
    current_indent: usize,
}

impl Default for Summary {
    fn default() -> Self {
        Self { value: "".to_string(), current_indent: 0 }
    }
}

impl Summary {
    pub fn summarize(value: &(impl Clone + Summarize)) -> String {
        let mut summary = Self::default();
        value.clone().summarize(&mut summary);
        summary.value
    }

    pub fn run(response: &Result<impl Clone + Summarize>) -> String {
        let mut summary = Self::default();
        if let Ok(v) = response {
            v.clone().summarize(&mut summary);
        } else {
            panic!("Error in response")
        }
        summary.value
    }

    pub fn primitive(&mut self, string: impl Debug) {
        self.value.push_str(&format!("{:?}", string));
    }

    pub fn child(&mut self, name: &'static str, value: Option<impl Summarize>) {
        if let Some(v) = value {
            self.child_node(name, v);
        }
    }

    pub fn child_node(&mut self, name: &'static str, value: impl Summarize) {
        self.child_node_indent(name, value, true);
    }

    pub fn child_node_indent(&mut self, name: &'static str, value: impl Summarize, indent: bool) {
        if indent {
            self.indent();
        }
        self.newline();
        self.value.push_str(name);
        self.value.push_str(": ");
        value.summarize(self);
        if indent {
            self.dedent();
        }
    }

    pub fn value(&mut self, value: Option<impl Summarize>) {
        if let Some(v) = value {
            v.summarize(self);
        }
    }

    pub fn value_node(&mut self, value: impl Summarize) {
        value.summarize(self);
    }

    pub fn children(&mut self, name: &'static str, children: Vec<impl Summarize>) {
        self.newline();
        self.value.push_str(name);
        self.value.push_str(": ");
        for child in children {
            child.summarize(self);
        }
    }

    pub fn values(&mut self, children: Vec<impl Summarize>) {
        for child in children {
            child.summarize(self);
        }
    }

    fn newline(&mut self) {
        let indent = "    ".repeat(self.current_indent);
        self.value.push('\n');
        self.value.push_str(&indent);
    }

    fn indent(&mut self) {
        self.current_indent += 1;
    }

    fn dedent(&mut self) {
        self.current_indent -= 1;
    }
}

impl Summarize for bool {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive(self)
    }
}

impl Summarize for u32 {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive(self)
    }
}

impl Summarize for String {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive(self)
    }
}

impl Summarize for PlayerName {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive(self)
    }
}

impl Summarize for PlayerSide {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive(self)
    }
}

impl Summarize for RoomIdentifier {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive(self)
    }
}

impl Summarize for CardIdentifier {
    fn summarize(self, summary: &mut Summary) {
        match adapters::server_card_id(self).expect("server_card_id") {
            ServerCardId::CardId(card_id) => summary.primitive(card_id),
            ServerCardId::AbilityId(ability_id) => summary.primitive(ability_id),
        }
    }
}

impl Summarize for GameObjectIdentifier {
    fn summarize(self, summary: &mut Summary) {
        summary.value(self.id)
    }
}

impl Summarize for Id {
    fn summarize(self, summary: &mut Summary) {
        match self {
            Id::CardId(id) => summary.value_node(id),
            _ => summary.primitive(self),
        }
    }
}

impl Summarize for SpriteAddress {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive("<SpriteAddress>")
    }
}

impl Summarize for AudioClipAddress {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive("<AudioClipAddress>")
    }
}

impl Summarize for EffectAddress {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive("<EffectAddress>")
    }
}

impl Summarize for ProjectileAddress {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive("<ProjectileAddress>")
    }
}

impl Summarize for PanelAddress {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive(self)
    }
}

impl Summarize for TimeValue {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive(self.milliseconds)
    }
}

impl Summarize for GameResponse {
    fn summarize(self, summary: &mut Summary) {
        summary.child_node_indent("command_list", self.command_list, false);
        if let Some((_, list)) = self.opponent_response {
            summary.child_node_indent("channel_response", list, false);
        }
    }
}

impl Summarize for CommandList {
    fn summarize(self, summary: &mut Summary) {
        summary.values(self.commands);
    }
}

impl Summarize for GameCommand {
    fn summarize(self, summary: &mut Summary) {
        summary.value(self.command);
    }
}

impl Summarize for Command {
    fn summarize(self, summary: &mut Summary) {
        match self {
            Command::Debug(_) => summary.primitive("Debug!"),
            Command::Delay(v) => summary.child_node("Delay", v),
            Command::ConnectToGame(v) => summary.child_node("ConnectToGame", v),
            Command::UpdatePanels(v) => summary.child_node("UpdatePanels", v),
            Command::TogglePanel(v) => summary.child_node("TogglePanel", v),
            Command::UpdateGameView(v) => summary.child_node("UpdateGameView", v),
            Command::VisitRoom(v) => summary.child_node("VisitRoom", v),
            Command::MoveGameObjects(v) => summary.child_node("MoveGameObjects", v),
            Command::PlaySound(v) => summary.child_node("PlaySound", v),
            Command::SetMusic(v) => summary.child_node("SetMusic", v),
            Command::FireProjectile(v) => summary.child_node("FireProjectile", v),
            Command::PlayEffect(v) => summary.child_node("PlayEffect", v),
            Command::DisplayGameMessage(v) => summary.child_node("DisplayGameMessage", v),
            Command::SetGameObjectsEnabled(v) => summary.child_node("SetGameObjectsEnabled", v),
            Command::DisplayRewards(v) => summary.child_node("DisplayRewards", v),
            Command::LoadScene(v) => summary.child_node("LoadScene", v),
            Command::SetPlayerId(v) => summary.child_node("SetPlayerId", v),
            Command::CreateTokenCard(v) => summary.child_node("CreateTokenCard", v),
        }
    }
}

impl Summarize for RunInParallelCommand {
    fn summarize(self, summary: &mut Summary) {
        summary.values(self.commands);
    }
}

impl Summarize for DelayCommand {
    fn summarize(self, summary: &mut Summary) {
        summary.value(self.duration)
    }
}

impl Summarize for ConnectToGameCommand {
    fn summarize(self, summary: &mut Summary) {
        summary.child_node("scene_name", self.scene_name);
    }
}

impl Summarize for TogglePanelCommand {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive("<TogglePanelCommand>");
    }
}

impl Summarize for UpdatePanelsCommand {
    fn summarize(self, summary: &mut Summary) {
        summary.children("panels", self.panels);
    }
}

impl Summarize for InterfacePanel {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive("<Panel>");
    }
}

impl Summarize for InterfaceMainControls {
    fn summarize(self, summary: &mut Summary) {
        summary.child("node", self.node);
        summary.children("card_anchor_nodes", self.card_anchor_nodes);
    }
}

impl Summarize for CardAnchorNode {
    fn summarize(self, summary: &mut Summary) {
        summary.child("card_id", self.card_id);
        summary.child("node", self.node);
        summary.children("anchors", self.anchors)
    }
}

impl Summarize for CardAnchor {
    fn summarize(self, summary: &mut Summary) {
        summary.child("node_corner", AnchorCorner::from_i32(self.node_corner));
        summary.child("card_corner", AnchorCorner::from_i32(self.card_corner));
    }
}

impl Summarize for AnchorCorner {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive(self)
    }
}

impl Summarize for Node {
    fn summarize(self, summary: &mut Summary) {
        if let Some(NodeType { node_type: Some(node_type::NodeType::Text(s)) }) = self.node_type {
            summary.child_node("text", s.label)
        }

        summary.values(self.children)
    }
}

impl Summarize for UpdateGameViewCommand {
    fn summarize(self, summary: &mut Summary) {
        summary.value(self.game);
    }
}

impl Summarize for GameView {
    fn summarize(self, summary: &mut Summary) {
        summary.child("user", self.user);
        summary.child("opponent", self.opponent);
        summary.child_node("raid_active", self.raid_active);
        summary.child("controls", self.main_controls);
        summary.child("game_object_positions", self.game_object_positions);
        summary.children("cards", self.cards);
    }
}

impl Summarize for PlayerView {
    fn summarize(self, summary: &mut Summary) {
        summary.child("side", PlayerSide::from_i32(self.side));
        summary.child("player_info", self.player_info);
        summary.child("mana", self.mana);
        summary.child("action_tracker", self.action_tracker);
        summary.child("score", self.score);
        summary.child_node("can_take_action", self.can_take_action);
    }
}

impl Summarize for PlayerInfo {
    fn summarize(self, summary: &mut Summary) {
        summary.child("name", self.name);
        summary.child("portrait", self.portrait);
        summary.child("portrait_frame", self.portrait_frame);
        summary.child("card_back", self.card_back);
        summary.children(
            "valid_rooms_to_visit",
            self.valid_rooms_to_visit
                .iter()
                .map(|i| RoomIdentifier::from_i32(*i).expect("RoomIdentifier"))
                .collect(),
        );
    }
}

impl Summarize for ManaView {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive(self.base_mana);
        if self.bonus_mana > 0 {
            summary.child_node("bonus_mana", self.bonus_mana);
        }
    }
}

impl Summarize for ActionTrackerView {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive(self.available_action_count);
    }
}

impl Summarize for ScoreView {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive(self.score);
    }
}

impl Summarize for GameObjectPositions {
    fn summarize(self, summary: &mut Summary) {
        summary.child("user_deck", self.user_deck);
        summary.child("opponent_deck", self.opponent_deck);
        summary.child("user_identity", self.user_identity);
        summary.child("opponent_identity", self.opponent_identity);
        summary.child("user_discard", self.user_discard);
        summary.child("opponent_discard", self.opponent_discard);
    }
}

impl Summarize for VisitRoomCommand {
    fn summarize(self, summary: &mut Summary) {
        summary.child("initiator", PlayerName::from_i32(self.initiator));
        summary.child("room_id", RoomIdentifier::from_i32(self.room_id));
        summary.child("visit_type", RoomVisitType::from_i32(self.visit_type));
    }
}

impl Summarize for RoomVisitType {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive(self)
    }
}

impl Summarize for CardCreationAnimation {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive(self)
    }
}

impl Summarize for CardView {
    fn summarize(self, summary: &mut Summary) {
        summary.child("card_id", self.card_id);
        summary.child_node("revealed_to_viewer", self.revealed_to_viewer);
        summary.child_node("is_face_up", self.is_face_up);
        summary.child("card_icons", self.card_icons);
        summary.child("arena_frame", self.arena_frame);
        summary.child("owning_player", PlayerName::from_i32(self.owning_player));
        summary.child("revealed_card", self.revealed_card);
    }
}

impl Summarize for CardIcons {
    fn summarize(self, summary: &mut Summary) {
        summary.child("top_left_icon", self.top_left_icon);
        summary.child("top_right_icon", self.top_right_icon);
        summary.child("bottom_left_icon", self.bottom_left_icon);
        summary.child("bottom_right_icon", self.bottom_right_icon);
        summary.child("arena_icon", self.arena_icon);
    }
}

impl Summarize for CardIcon {
    fn summarize(self, summary: &mut Summary) {
        if let Some(text) = self.text {
            summary.primitive(text);
        }
    }
}

impl Summarize for RevealedCardView {
    fn summarize(self, summary: &mut Summary) {
        summary.child("card_frame", self.card_frame);
        summary.child("title_background", self.title_background);
        summary.child("jewel", self.jewel);
        summary.child("image", self.image);
        summary.child("title", self.title);
        summary.child("rules_text", self.rules_text);
        summary.child("targeting", self.targeting);
        summary.child("on_release_position", self.on_release_position);
    }
}

impl Summarize for CardTitle {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive(self.text)
    }
}

impl Summarize for RulesText {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive("<RulesText>");
    }
}

impl Summarize for CardTargeting {
    fn summarize(self, summary: &mut Summary) {
        summary.value(self.targeting);
    }
}

impl Summarize for Targeting {
    fn summarize(self, summary: &mut Summary) {
        match self {
            Targeting::NoTargeting(NoTargeting { can_play }) => {
                summary.child_node("can_play", can_play)
            }
            Targeting::PlayInRoom(PlayInRoom { valid_rooms }) => {
                summary.children(
                    "valid_rooms",
                    valid_rooms
                        .iter()
                        .map(|i| RoomIdentifier::from_i32(*i).expect("RoomIdentifier"))
                        .collect(),
                );
            }
            Targeting::ArrowTargetRoom(ArrowTargetRoom { valid_rooms, .. }) => {
                summary.children(
                    "valid_rooms",
                    valid_rooms
                        .iter()
                        .map(|i| RoomIdentifier::from_i32(*i).expect("RoomIdentifier"))
                        .collect(),
                );
            }
        }
    }
}

impl Summarize for ObjectPosition {
    fn summarize(self, summary: &mut Summary) {
        summary.child_node("sorting_key", self.sorting_key);
        summary.child("position", self.position);
    }
}

impl Summarize for Position {
    fn summarize(self, summary: &mut Summary) {
        match self {
            Position::Offscreen(v) => summary.primitive(v),
            Position::Room(v) => summary.primitive(v),
            Position::Item(v) => summary.primitive(v),
            Position::Staging(v) => summary.primitive(v),
            Position::Hand(v) => summary.primitive(v),
            Position::Deck(v) => summary.primitive(v),
            Position::DeckContainer(v) => summary.primitive(v),
            Position::DiscardPile(v) => summary.primitive(v),
            Position::DiscardPileContainer(v) => summary.primitive(v),
            Position::Raid(v) => summary.primitive(v),
            Position::Browser(v) => summary.primitive(v),
            Position::Identity(v) => summary.primitive(v),
            Position::IdentityContainer(v) => summary.primitive(v),
            Position::IntoCard(v) => summary.primitive(v),
            Position::Revealed(v) => summary.primitive(v),
        }
    }
}

impl Summarize for PlaySoundCommand {
    fn summarize(self, summary: &mut Summary) {
        summary.child("sound", self.sound);
    }
}

impl Summarize for SetMusicCommand {
    fn summarize(self, summary: &mut Summary) {
        summary.child("music_state", MusicState::from_i32(self.music_state));
    }
}

impl Summarize for MusicState {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive(self)
    }
}

impl Summarize for FireProjectileCommand {
    fn summarize(self, summary: &mut Summary) {
        summary.child("source_id", self.source_id);
        summary.child("target_id", self.target_id);
        summary.child("projectile", self.projectile);
        summary.child("travel_duration", self.travel_duration);
        summary.child("fire_sound", self.fire_sound);
        summary.child("impact_sound", self.impact_sound);
        summary.child("additional_hit", self.additional_hit);
        summary.child("additional_hit_delay", self.additional_hit_delay);
        summary.child("wait_duration", self.wait_duration);
        summary.child("jump_to_position", self.jump_to_position);
    }
}

impl Summarize for PlayEffectCommand {
    fn summarize(self, summary: &mut Summary) {
        summary.child("effect", self.effect);
        if let Some(PlayEffectPosition { effect_position: Some(EffectPosition::GameObject(id)) }) =
            self.position
        {
            summary.child_node("position", id);
        }
        summary.child("duration", self.duration);
        summary.child("sound", self.sound);
    }
}

impl Summarize for DisplayGameMessageCommand {
    fn summarize(self, summary: &mut Summary) {
        summary.value(GameMessageType::from_i32(self.message_type));
    }
}

impl Summarize for GameMessageType {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive(self)
    }
}

impl Summarize for SetGameObjectsEnabledCommand {
    fn summarize(self, summary: &mut Summary) {
        summary.child_node("game_objects_enabled", self.game_objects_enabled)
    }
}

impl Summarize for DisplayRewardsCommand {
    fn summarize(self, summary: &mut Summary) {
        summary.values(self.rewards);
    }
}

impl Summarize for LoadSceneCommand {
    fn summarize(self, summary: &mut Summary) {
        summary.child_node("scene_name", self.scene_name);
        summary.child("mode", SceneLoadMode::from_i32(self.mode))
    }
}

impl Summarize for SceneLoadMode {
    fn summarize(self, summary: &mut Summary) {
        summary.primitive(self)
    }
}

impl Summarize for SetPlayerIdentifierCommand {
    fn summarize(self, _: &mut Summary) {}
}

impl Summarize for GameObjectMove {
    fn summarize(self, summary: &mut Summary) {
        summary.child("id", self.id);
        summary.child("position", self.position);
    }
}

impl Summarize for MoveGameObjectsCommand {
    fn summarize(self, summary: &mut Summary) {
        summary.values(self.moves);
    }
}

impl Summarize for CreateTokenCardCommand {
    fn summarize(self, summary: &mut Summary) {
        summary.child("card", self.card);
    }
}
