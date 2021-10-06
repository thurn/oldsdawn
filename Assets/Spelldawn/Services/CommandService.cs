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
using Spelldawn.Masonry;
using Spelldawn.Protos;
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class CommandService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;

    public IEnumerator<YieldInstruction> HandleCommands(CommandList commandList)
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
            break;
          case GameCommand.CommandOneofCase.CreateCard:
            yield return StartCoroutine(_registry.CardService.CreateCard(command.CreateCard));
            break;
          case GameCommand.CommandOneofCase.UpdateCard:
            break;
          case GameCommand.CommandOneofCase.MoveCard:
            yield return StartCoroutine(_registry.CardService.MoveCard(command.MoveCard));
            break;
          case GameCommand.CommandOneofCase.DestroyCard:
            break;
          case GameCommand.CommandOneofCase.UpdatePlayerState:
            break;
          case GameCommand.CommandOneofCase.CreateOrUpdateRoom:
            break;
          case GameCommand.CommandOneofCase.DestroyRoom:
            break;
          case GameCommand.CommandOneofCase.None:
          default:
            break;
        }
      }
    }

    void HandleRenderInterface(RenderInterfaceCommand command)
    {
      var rootElement = _registry.Document.rootVisualElement;
      rootElement.Clear();
      if (command.Node != null)
      {
        rootElement.Add(Mason.Render(_registry, command.Node));
      }
    }

    IEnumerator<YieldInstruction> HandleRenderGame(RenderGameCommand command)
    {
      _registry.CardService.Initialize(
        command.Game?.User?.PlayerInfo?.IdentityCard);
      yield break;
    }
  }
}