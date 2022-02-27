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
using UnityEngine.SceneManagement;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class CommandService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] bool _currentlyHandling;
    readonly Queue<CommandList> _queue = new();

    public bool CurrentlyHandlingCommand => _currentlyHandling;

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
          case GameCommand.CommandOneofCase.Debug:
            HandleClientDebugCommand(command.Debug);
            break;
          case GameCommand.CommandOneofCase.RunInParallel:
            yield return HandleRunInParallel(command.RunInParallel);
            break;
          case GameCommand.CommandOneofCase.ConnectToGame:
            yield return ConnectToGame(command.ConnectToGame);
            break;
          case GameCommand.CommandOneofCase.RenderInterface:
            HandleRenderInterface(command.RenderInterface);
            break;
          case GameCommand.CommandOneofCase.TogglePanel:
            _registry.DocumentService.TogglePanel(
              command.TogglePanel.Open, command.TogglePanel.PanelAddress);
            break;
          case GameCommand.CommandOneofCase.Delay:
            yield return new WaitForSeconds(DataUtils.ToSeconds(command.Delay.Duration, 0));
            break;
          case GameCommand.CommandOneofCase.UpdateGameView:
            yield return HandleUpdateGameView(command.UpdateGameView.Game);
            break;
          case GameCommand.CommandOneofCase.VisitRoom:
            yield return _registry.ArenaService.HandleVisitRoom(command.VisitRoom);
            break;
          case GameCommand.CommandOneofCase.CreateOrUpdateCard:
            yield return _registry.CardService.HandleCreateOrUpdateCardCommand(command.CreateOrUpdateCard);
            break;
          case GameCommand.CommandOneofCase.DestroyCard:
            yield return _registry.CardService.HandleDestroyCard(command.DestroyCard.CardId);
            break;
          case GameCommand.CommandOneofCase.MoveGameObjects:
            yield return _registry.ObjectPositionService.HandleMoveGameObjectsCommand(command.MoveGameObjects);
            break;
          case GameCommand.CommandOneofCase.MoveObjectsAtPosition:
            yield return _registry.ObjectPositionService.HandleMoveGameObjectsAtPosition(command.MoveObjectsAtPosition);
            break;
          case GameCommand.CommandOneofCase.PlaySound:
            _registry.MainAudioSource.PlayOneShot(_registry.AssetService.GetAudioClip(command.PlaySound.Sound));
            break;
          case GameCommand.CommandOneofCase.SetMusic:
            _registry.MusicService.SetMusicState(command.SetMusic.MusicState);
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
          case GameCommand.CommandOneofCase.LoadScene:
            yield return HandleLoadScene(command.LoadScene);
            break;
          case GameCommand.CommandOneofCase.SetPlayerId:
            _registry.GameService.PlayerId = command.SetPlayerId.Id;
            break;
          case GameCommand.CommandOneofCase.None:
          default:
            break;
        }
      }

      onComplete?.Invoke();
    }

    IEnumerator ConnectToGame(ConnectToGameCommand command)
    {
      _registry.GameService.CurrentGameId = command.GameId;
      yield return SceneManager.LoadSceneAsync(command.SceneName, LoadSceneMode.Single);
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

      if (command.Sound != null)
      {
        _registry.MainAudioSource.PlayOneShot(_registry.AssetService.GetAudioClip(command.Sound));
      }

      yield return new WaitForSeconds(DataUtils.ToSeconds(command.Duration, 0));
    }

    IEnumerator HandleUpdateGameView(GameView game)
    {
      if (game.GameId != null)
      {
        _registry.GameService.CurrentGameId = game.GameId;
      }

      _registry.CardService.SetCardBacks(game.User?.PlayerInfo?.CardBack, game.Opponent?.PlayerInfo?.CardBack);

      if (game.CurrentPriority != PlayerName.Unspecified)
      {
        _registry.ActionService.CurrentPriority = game.CurrentPriority;
      }

      if (game.User != null)
      {
        if (game.User.Side != PlayerSide.Unspecified)
        {
          _registry.ArenaService.UpdateViewForSide(game.User.Side);
        }

        yield return HandleRenderPlayer(PlayerName.User, game.User);
      }

      if (game.Opponent != null)
      {
        yield return HandleRenderPlayer(PlayerName.Opponent, game.Opponent);
      }

      _registry.RaidService.RaidActive = game.RaidActive;
    }

    IEnumerator HandleRenderPlayer(PlayerName playerName, PlayerView playerView)
    {
      if (playerView.Side != PlayerSide.Unspecified)
      {
        _registry.IdentityCardForPlayer(playerName).Side = playerView.Side;
      }

      if (playerView.PlayerInfo != null)
      {
        yield return _registry.IdentityCardForPlayer(playerName).RenderPlayerInfo(playerView.PlayerInfo);
      }

      if (playerView.Score != null)
      {
        yield return _registry.IdentityCardForPlayer(playerName).RenderScore(playerView.Score);
      }

      if (playerView.Mana != null)
      {
        _registry.ManaDisplayForPlayer(playerName).RenderManaDisplay(playerView.Mana);
      }

      if (playerView.ActionTracker != null)
      {
        _registry.ActionDisplayForPlayer(playerName).RenderActionTrackerView(playerView.ActionTracker);
      }
    }

    IEnumerator HandleLoadScene(LoadSceneCommand command)
    {
      yield return SceneManager.LoadSceneAsync(command.SceneName, command.Mode switch
      {
        SceneLoadMode.Single => LoadSceneMode.Single,
        SceneLoadMode.Additive => LoadSceneMode.Additive,
        _ => LoadSceneMode.Single
      });
    }

    void HandleClientDebugCommand(ClientDebugCommand command)
    {
      switch (command.DebugCommandCase)
      {
        case ClientDebugCommand.DebugCommandOneofCase.ShowLogs:
          _registry.LogViewer.DoShow();
          break;
        case ClientDebugCommand.DebugCommandOneofCase.InvokeAction:
          // TODO: Enqueue actions
          StartCoroutine(HandleActionAsync(command.InvokeAction));
          break;
        case ClientDebugCommand.DebugCommandOneofCase.LogMessage:
          Debug.Log(command.LogMessage);
          break;
      }
    }

    IEnumerator HandleActionAsync(GameAction action)
    {
      yield return new WaitForSeconds(0.5f);
      _registry.ActionService.HandleAction(action);
    }
  }
}