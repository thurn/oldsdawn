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
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class ActionService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    bool _currentlyHandlingAction;

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
  }
}