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
using System.Collections.Generic;
using System.Linq;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class CommandService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] bool _currentlyHandling;
    readonly Queue<CommandList> _queue = new();

    public IEnumerator HandleCommands(IEnumerable<GameCommand> commands)
    {
      var list = new CommandList();
      list.Commands.AddRange(commands);
      return HandleCommands(list);
    }

    public IEnumerator HandleCommands(params GameCommand[] commands)
    {
      return HandleCommands(commands.ToList());
    }

    public IEnumerator HandleCommands(CommandList commandList)
    {
      _queue.Enqueue(commandList);
      return new WaitUntil(() => _currentlyHandling == false && _queue.Count == 0);
    }

    void Update()
    {
      if (_queue.Count > 0 && !_currentlyHandling)
      {
        _currentlyHandling = true;
        StartCoroutine(HandleCommandsAsync(_queue.Dequeue(), () => { _currentlyHandling = false; }));
      }
    }

    IEnumerator HandleCommandsAsync(CommandList commandList, Action? onComplete = null)
    {
      yield return _registry.AssetService.LoadAssets(commandList);

      foreach (var command in commandList.Commands)
      {
        switch (command.CommandCase)
        {
          case GameCommand.CommandOneofCase.DebugLog:
            Debug.Log(command.DebugLog.Message);
            break;
          case GameCommand.CommandOneofCase.RunInParallel:
            yield return HandleRunInParallel(command.RunInParallel);
            break;
          case GameCommand.CommandOneofCase.RenderInterface:
            HandleRenderInterface(command.RenderInterface);
            break;
          case GameCommand.CommandOneofCase.Delay:
            yield return new WaitForSeconds(DataUtils.ToSeconds(command.Delay.Duration, 0));
            break;
          case GameCommand.CommandOneofCase.RenderGame:
            yield return HandleRenderGame(command.RenderGame.Game);
            break;
          case GameCommand.CommandOneofCase.InitiateRaid:
            yield return _registry.RaidService.HandleInitiateRaid(command.InitiateRaid);
            break;
          case GameCommand.CommandOneofCase.EndRaid:
            yield return _registry.RaidService.HandleEndRaid(command.EndRaid);
            break;
          case GameCommand.CommandOneofCase.LevelUpRoom:
            yield return _registry.ArenaService.HandleLevelUpRoom(command.LevelUpRoom);
            break;
          case GameCommand.CommandOneofCase.CreateCard:
            yield return _registry.ObjectPositionService.HandleCreateCardCommand(command.CreateCard);
            break;
          case GameCommand.CommandOneofCase.UpdateCard:
            yield return _registry.ObjectPositionService.HandleUpdateCardCommand(command.UpdateCard);
            break;
          case GameCommand.CommandOneofCase.MoveGameObjects:
            yield return _registry.ObjectPositionService.HandleMoveGameObjectsCommand(command.MoveGameObjects);
            break;
          case GameCommand.CommandOneofCase.DestroyCard:
            break;
          case GameCommand.CommandOneofCase.UpdatePlayerState:
            break;
          case GameCommand.CommandOneofCase.FireProjectile:
            yield return
              _registry.ObjectPositionService.HandleFireProjectileCommand(command.FireProjectile);
            break;
          case GameCommand.CommandOneofCase.PlayEffect:
            yield return HandlePlayEffect(command.PlayEffect);
            break;
          case GameCommand.CommandOneofCase.DisplayGameMessage:
            yield return _registry.GameMessage.Show(command.DisplayGameMessage);
            break;
          case GameCommand.CommandOneofCase.SetGameObjectsEnabled:
            yield return _registry.ArenaService.HandleSetGameObjectsEnabled(command.SetGameObjectsEnabled);
            break;
          case GameCommand.CommandOneofCase.DisplayRewards:
            yield return _registry.RewardChest.HandleDisplayRewards(command.DisplayRewards);
            break;
          case GameCommand.CommandOneofCase.None:
          default:
            break;
        }
      }

      onComplete?.Invoke();
    }

    IEnumerator HandleRunInParallel(RunInParallelCommand command)
    {
      var coroutines = new List<Coroutine>();
      foreach (var list in command.Commands)
      {
        coroutines.Add(StartCoroutine(HandleCommandsAsync(list)));
      }

      foreach (var coroutine in coroutines)
      {
        yield return coroutine;
      }
    }

    void HandleRenderInterface(RenderInterfaceCommand command)
    {
      _registry.DocumentService.HandleRenderInterface(command);
    }

    IEnumerator HandleRenderGame(GameView game)
    {
      _registry.ObjectPositionService.Initialize(
        game.User?.PlayerInfo?.IdentityCard,
        game.Opponent?.PlayerInfo?.IdentityCard);

      _registry.ArenaService.SetRoomsOnBottom(game.Arena?.RoomsAtBottom == true);

      if (game.CurrentPriority != PlayerName.Unspecified)
      {
        _registry.ActionService.CurrentPriority = game.CurrentPriority;
      }

      _registry.IdentityCardForPlayer(PlayerName.User).SetScore(game.User?.Score?.Score);
      _registry.IdentityCardForPlayer(PlayerName.User).IdentityAction = game.Arena?.IdentityAction;
      yield break;
    }

    IEnumerator HandlePlayEffect(PlayEffectCommand command)
    {
      var position = command.Position.EffectPositionCase switch
      {
        PlayEffectPosition.EffectPositionOneofCase.GameObject =>
          _registry.ObjectPositionService.Find(command.Position.GameObject).transform.position,
        _ => throw new ArgumentOutOfRangeException()
      };

      var rotation = Quaternion.LookRotation(position - _registry.MainCamera.transform.position);
      var effect = _registry.AssetPoolService.Create(_registry.AssetService.GetEffect(command.Effect), position);
      effect.transform.rotation = rotation;
      if (command.Scale is { } scale)
      {
        effect.transform.localScale = scale * Vector3.one;
      }

      yield return new WaitForSeconds(DataUtils.ToSeconds(command.Duration, 0));
    }
  }
}