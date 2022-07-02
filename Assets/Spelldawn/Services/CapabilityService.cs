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

#nullable enable

using Spelldawn.Game;
using Spelldawn.Protos;
using UnityEngine;

namespace Spelldawn.Services
{
  public sealed class CapabilityService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;

    public PlayerName CurrentPriority { get; set; }

    /// <summary>
    /// Can the user *start* performing an action such as dragging a card out of their hand or dragging a raid arrow.
    /// This is allowed more leniently than actually *performing* an action as defined by
    /// <see cref="CanExecuteAction"/> below.
    /// </summary>
    public bool CanInitiateAction() => !_registry.CardService.CurrentlyDragging &&
                                       !AnyOverlayOpen() &&
                                       !_registry.DocumentService.IsAnyPanelOpen();

    public bool AnyOverlayOpen() => _registry.RaidOverlay.Enabled ||
                                    _registry.InterfaceOverlay.Enabled ||
                                    _registry.LongPressOverlay.Enabled;


    /// <summary>
    /// Can the user currently zoom a card that exists in the provided GameContext.
    /// </summary>
    public bool CanInfoZoom(Displayable displayable, GameContext gameContext)
    {
      if (_registry.DocumentService.IsAnyPanelOpen())
      {
        return false;
      }

      switch (gameContext)
      {
        case GameContext.ArenaRaidParticipant:
        case GameContext.RaidParticipant:
          // If a card is a top-level raid participant, it can be info zoomed. However if a card is *part* of
          // a parent display that is participating in a raid (e.g. it is part of the discard pile that is 
          // being targeted), then it cannot be info zoomed and the long-press browser is used instead.
          return displayable.Parent == _registry.RaidService.RaidParticipants ||
                 displayable.Parent == _registry.ArenaService.LeftItems ||
                 displayable.Parent == _registry.ArenaService.RightIems;
        case GameContext.Browser:
        case GameContext.RewardBrowser:
        case GameContext.LongPressBrowser:
        case GameContext.RevealedCardsBrowser:
          return true;
        case GameContext.Deck:
        case GameContext.DiscardPile:
          return false;
        default:
          return !AnyOverlayOpen();
      }
    }


    /// <summary>
    /// Can the user currently perform a game action of the provided type?
    /// </summary>
    public bool CanExecuteAction(GameAction.ActionOneofCase actionType) => actionType switch
    {
      GameAction.ActionOneofCase.StandardAction => CanAct(
        allowInOverlay: true,
        actionPointRequired: false,
        allowWithPanelOpen: true),
      GameAction.ActionOneofCase.FetchPanel => true,
      GameAction.ActionOneofCase.CreateNewGame => true,
      GameAction.ActionOneofCase.GainMana => CanAct(),
      GameAction.ActionOneofCase.DrawCard => CanAct(),
      GameAction.ActionOneofCase.PlayCard => CanAct(),
      GameAction.ActionOneofCase.LevelUpRoom => CanAct(),
      GameAction.ActionOneofCase.InitiateRaid => CanAct(),
      _ => false
    };

    bool CanAct(bool allowInOverlay = false, bool actionPointRequired = true, bool allowWithPanelOpen = false) =>
      !_registry.CardService.CurrentlyDragging &&
      (allowWithPanelOpen || !_registry.DocumentService.IsAnyPanelOpen()) &&
      (allowInOverlay || !AnyOverlayOpen()) &&
      (allowInOverlay || !_registry.RaidService.RaidActive) &&
      (!actionPointRequired || _registry.ActionDisplayForPlayer(PlayerName.User).AvailableActions > 0);

    void Update()
    {
      var userLight = _registry.ActiveLightForPlayer(PlayerName.User);
      var opponentLight = _registry.ActiveLightForPlayer(PlayerName.Opponent);

      switch (_registry.CapabilityService.CurrentPriority)
      {
        case PlayerName.User when CanExecuteAction(GameAction.ActionOneofCase.PlayCard):
          userLight.SetActive(true);
          break;
        case PlayerName.Opponent:
          opponentLight.SetActive(true);
          break;
        case PlayerName.Unspecified:
        default:
          userLight.SetActive(false);
          opponentLight.SetActive(false);
          break;
      }
    }
  }
}