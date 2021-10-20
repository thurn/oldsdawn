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

using System.Collections.Generic;
using Spelldawn.Protos;
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class CommandService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    bool _currentlyHandling;
    readonly Queue<CommandList> _queue = new();

    public void HandleCommands(params GameCommand[] commands)
    {
      var list = new CommandList();
      list.Commands.AddRange(commands);
      HandleCommands(list);
    }

    public void HandleCommands(CommandList commandList)
    {
      _queue.Enqueue(commandList);
    }

    void Update()
    {
      if (_queue.Count > 0 && !_currentlyHandling)
      {
        _currentlyHandling = true;
        StartCoroutine(HandleCommandsAsync(_queue.Dequeue()));
      }
    }

    IEnumerator<YieldInstruction> HandleCommandsAsync(CommandList commandList)
    {
      yield return StartCoroutine(_registry.AssetService.LoadAssets(commandList));

      foreach (var command in commandList.Commands)
      {
        switch (command.CommandCase)
        {
          case GameCommand.CommandOneofCase.RenderInterface:
            HandleRenderInterface(command.RenderInterface);
            break;
          case GameCommand.CommandOneofCase.RenderGame:
            yield return StartCoroutine(HandleRenderGame(command.RenderGame));
            break;
          case GameCommand.CommandOneofCase.InitiateRaid:
            yield return StartCoroutine(_registry.RaidService.HandleInitiateRaid(command.InitiateRaid));
            break;
          case GameCommand.CommandOneofCase.EndRaid:
            break;
          case GameCommand.CommandOneofCase.CreateCard:
            yield return StartCoroutine(_registry.CardService.HandleCreateCardCommand(command.CreateCard));
            break;
          case GameCommand.CommandOneofCase.UpdateCard:
            break;
          case GameCommand.CommandOneofCase.MoveCard:
            yield return StartCoroutine(_registry.CardService.HandleMoveCardCommand(command.MoveCard));
            break;
          case GameCommand.CommandOneofCase.DestroyCard:
            break;
          case GameCommand.CommandOneofCase.UpdatePlayerState:
            break;
          case GameCommand.CommandOneofCase.FireProjectile:
            yield return StartCoroutine(_registry.CardService.HandleFireProjectileCommand(command.FireProjectile));
            break;
          case GameCommand.CommandOneofCase.None:
          default:
            break;
        }
      }

      _currentlyHandling = false;
    }

    void HandleRenderInterface(RenderInterfaceCommand command)
    {
      _registry.DocumentService.HandleRenderInterface(command);
    }

    IEnumerator<YieldInstruction> HandleRenderGame(RenderGameCommand command)
    {
      _registry.CardService.Initialize(
        command.Game?.User?.PlayerInfo?.IdentityCard,
        command.Game?.Opponent?.PlayerInfo?.IdentityCard);
      yield break;
    }
  }
}