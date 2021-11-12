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

using System.Collections;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class ActionService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] PlayerName _currentPriority;
    [SerializeField] bool _currentlyHandlingAction;

    public bool CurrentlyHandlingAction => _currentlyHandlingAction;

    public PlayerName CurrentPriority
    {
      set => _currentPriority = value;
    }

    public void HandleAction(GameAction action)
    {
      if (_currentlyHandlingAction)
      {
        Debug.LogError($"Error: Already handling action, cannot process {action}");
        return;
      }

      _currentlyHandlingAction = true;
      StartCoroutine(HandleActionAsync(action));
    }

    void Update()
    {
      var userLight = _registry.ActiveLightForPlayer(PlayerName.User);
      var opponentLight = _registry.ActiveLightForPlayer(PlayerName.Opponent);
      var canShow = !_currentlyHandlingAction && !_registry.RaidService.RaidActive;

      switch (_currentPriority)
      {
        case PlayerName.User when canShow && _registry.ActionDisplayForPlayer(PlayerName.User).AvailableActions > 0:
          userLight.SetActive(true);
          break;
        case PlayerName.Opponent when canShow:
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
          if (action.StandardAction.OptimisticUpdate is { } response)
          {
            yield return _registry.CommandService.HandleCommands(response);
          }

          break;
        case GameAction.ActionOneofCase.DrawCard:
          _registry.ObjectPositionService.DrawOptimisticCard();
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
      var card = _registry.ObjectPositionService.FindCard(action.CardId);
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