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

using System;
using System.Collections;
using System.Text;
using Spelldawn.Game;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEngine;
using Random = UnityEngine.Random;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class ActionService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] PlayerName _currentPriority;
    [SerializeField] bool _currentlyHandlingAction;

    public PlayerName CurrentPriority
    {
      set => _currentPriority = value;
    }

    public void HandleAction(GameAction action)
    {
      if (!CanPerformAction(action.ActionCase))
      {
        var message = new StringBuilder();
        message.Append($"Error: User cannot currently perform action {action}");
        message.Append($"\nCurrently Handling Action: {_currentlyHandlingAction}");
        throw new InvalidOperationException(message.ToString());
      }

      _currentlyHandlingAction = true;
      StartCoroutine(HandleActionAsync(action));
    }

    public bool CanInfoZoom(GameContext gameContext)
    {
      switch (gameContext)
      {
        case GameContext.ArenaRaidParticipant:
        case GameContext.RaidParticipant:
        case GameContext.Browser:
        case GameContext.RewardBrowser:
          return true;
        default:
          return !_registry.BackgroundOverlay.Enabled;
      }
    }

    public bool CanInitiateAction() => _registry.CardService.CurrentlyDragging == null &&
                                     !_registry.BackgroundOverlay.Enabled;

    public bool CanPerformAction(GameAction.ActionOneofCase actionType) => actionType switch
    {
      GameAction.ActionOneofCase.StandardAction => CanAct(allowInOverlay: true, actionPointRequired: false),
      GameAction.ActionOneofCase.GainMana => CanAct(),
      GameAction.ActionOneofCase.DrawCard => CanAct(),
      GameAction.ActionOneofCase.PlayCard => CanAct(),
      GameAction.ActionOneofCase.LevelUpRoom => CanAct(),
      GameAction.ActionOneofCase.InitiateRaid => CanAct(),
      _ => false
    };

    bool CanAct(bool allowInOverlay = false, bool actionPointRequired = true) =>
      !_currentlyHandlingAction &&
      !_registry.CommandService.CurrentlyHandlingCommand &&
      _registry.CardService.CurrentlyDragging == null &&
      (allowInOverlay || !_registry.BackgroundOverlay.Enabled) &&
      (allowInOverlay || !_registry.RaidService.RaidActive) &&
      (!actionPointRequired || _registry.ActionDisplayForPlayer(PlayerName.User).AvailableActions > 0);

    void Update()
    {
      var userLight = _registry.ActiveLightForPlayer(PlayerName.User);
      var opponentLight = _registry.ActiveLightForPlayer(PlayerName.Opponent);

      switch (_currentPriority)
      {
        case PlayerName.User when CanPerformAction(GameAction.ActionOneofCase.PlayCard):
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

    IEnumerator HandleActionAsync(GameAction action)
    {
      yield return ApplyOptimisticResponse(action);

      // Send to server
      yield return new WaitForSeconds(Random.Range(0.1f, 1f) + (Random.Range(0f, 1f) < 0.1f ? 1f : 0));

      yield return _registry.SampleData.FakeActionResponse(action);
      _currentlyHandlingAction = false;
    }

    IEnumerator ApplyOptimisticResponse(GameAction action)
    {
      switch (action.ActionCase)
      {
        case GameAction.ActionOneofCase.StandardAction:
          if (action.StandardAction.Update is { } update)
          {
            yield return _registry.CommandService.HandleCommands(update);
          }

          break;
        case GameAction.ActionOneofCase.DrawCard:
          _registry.CardService.DrawOptimisticCard();
          break;
        case GameAction.ActionOneofCase.PlayCard:
          yield return HandlePlayCard(action.PlayCard);
          break;
        case GameAction.ActionOneofCase.GainMana:
          _registry.ManaDisplayForPlayer(PlayerName.User).Increment();
          break;
        case GameAction.ActionOneofCase.InitiateRaid:
          yield return _registry.CommandService.HandleCommands(new GameCommand
          {
            InitiateRaid = new InitiateRaidCommand
            {
              RoomId = action.InitiateRaid.RoomId,
              Initiator = PlayerName.User
            }
          });
          break;
        case GameAction.ActionOneofCase.LevelUpRoom:
          yield return _registry.CommandService.HandleCommands(new GameCommand
          {
            LevelUpRoom = new LevelUpRoomCommand
            {
              RoomId = action.LevelUpRoom.RoomId,
              Initiator = PlayerName.User
            }
          });
          break;
        default:
          yield break;
      }
    }

    IEnumerator HandlePlayCard(PlayCardAction action)
    {
      var card = _registry.CardService.FindCard(action.CardId);
      var position =
        Errors.CheckNotNull(card.RevealedCardView?.OnReleasePosition, "Card does not have release position");
      if (position.PositionCase == ObjectPosition.PositionOneofCase.Room)
      {
        if (card.TargetRoom is { } targetRoom)
        {
          // Move to targeted room if one is available
          var newPosition = new ObjectPosition();
          newPosition.MergeFrom(position);
          newPosition.Room.RoomId = targetRoom;
          position = newPosition;
        }
      }

      return _registry.ObjectPositionService.MoveGameObject(card, position);
    }
  }
}