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
using System.Net.Http;
using System.Text;
using Grpc.Core;
using Grpc.Net.Client;
using Grpc.Net.Client.Web;
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
    readonly RaycastHit[] _raycastHitsTempBuffer = new RaycastHit[8];

    readonly Protos.Spelldawn.SpelldawnClient _client = new(GrpcChannel.ForAddress(
      "http://localhost:50052", new GrpcChannelOptions
      {
        HttpHandler = new GrpcWebHandler(new HttpClientHandler()),
        Credentials = ChannelCredentials.Insecure
      }));

    [SerializeField] bool _fakeActionResponse;
    [SerializeField] Registry _registry = null!;
    [SerializeField] PlayerName _currentPriority;
    [SerializeField] bool _currentlyHandlingAction;
    Clickable? _lastClicked;

    public PlayerName CurrentPriority
    {
      set => _currentPriority = value;
    }

    public void Connect()
    {
      if (!_fakeActionResponse)
      {
        ConnectToServer();
      }
    }

    public void HandleAction(GameAction action)
    {
      if (!CanExecuteAction(action.ActionCase))
      {
        var message = new StringBuilder();
        message.Append($"Error: User cannot currently perform action {action}");
        message.Append($"\nCurrently Handling Action: {_currentlyHandlingAction}");
        throw new InvalidOperationException(message.ToString());
      }

      _currentlyHandlingAction = true;
      StartCoroutine(HandleActionAsync(action));
    }

    /// <summary>
    /// Can the user currently zoom a card that exists in the provided GameContext.
    /// </summary>
    public bool CanInfoZoom(GameContext gameContext)
    {
      switch (gameContext)
      {
        case GameContext.ArenaRaidParticipant:
        case GameContext.RaidParticipant:
        case GameContext.Browser:
        case GameContext.RewardBrowser:
          return true;
        case GameContext.Deck:
        case GameContext.DiscardPile:
          return false;
        default:
          return !_registry.BackgroundOverlay.Enabled;
      }
    }

    /// <summary>
    /// Can the user *start* performing an action such as dragging a card out of their hand or dragging a raid arrow.
    /// This is allowed more leniently than actually *performing* an action as defined by
    /// <see cref="CanExecuteAction"/> below.
    /// </summary>
    public bool CanInitiateAction() => !_registry.CardService.CurrentlyDragging &&
                                       !_registry.BackgroundOverlay.Enabled;

    /// <summary>
    /// Can the user currently perform a game action of the provided type?
    /// </summary>
    public bool CanExecuteAction(GameAction.ActionOneofCase actionType) => actionType switch
    {
      GameAction.ActionOneofCase.StandardAction => CanAct(allowInOverlay: true, actionPointRequired: false),
      GameAction.ActionOneofCase.TogglePanel => true,
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
      !_registry.CardService.CurrentlyDragging &&
      (allowInOverlay || !_registry.BackgroundOverlay.Enabled) &&
      (allowInOverlay || !_registry.RaidService.RaidActive) &&
      (!actionPointRequired || _registry.ActionDisplayForPlayer(PlayerName.User).AvailableActions > 0);

    void Update()
    {
      var userLight = _registry.ActiveLightForPlayer(PlayerName.User);
      var opponentLight = _registry.ActiveLightForPlayer(PlayerName.Opponent);

      switch (_currentPriority)
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

      switch (Input.GetMouseButton(0))
      {
        case true when _lastClicked:
          _lastClicked!.MouseDrag();
          break;
        case true when !_lastClicked:
          _lastClicked = FireMouseDown();
          break;
        case false when _lastClicked:
          var last = _lastClicked;
          _lastClicked = null; // Do this first in case MouseUp() throws
          last!.MouseUp();
          break;
      }
    }

    Clickable? FireMouseDown()
    {
      var ray = _registry.MainCamera.ScreenPointToRay(Input.mousePosition);
      var hits = Physics.RaycastNonAlloc(ray, _raycastHitsTempBuffer, 100);
      Clickable? fired = null;

      for (var i = 0; i < hits; ++i)
      {
        var hit = _raycastHitsTempBuffer[i];
        var clickable = hit.collider.GetComponent<Clickable>();
        if (clickable)
        {
          if (fired)
          {
            Debug.LogWarning($"Ignoring click on {clickable}, already handled click on {fired}");
          }
          else
          {
            var consumed = clickable.MouseDown();
            if (consumed)
            {
              fired = clickable;
            }
          }
        }
      }

      Array.Clear(_raycastHitsTempBuffer, 0, _raycastHitsTempBuffer.Length);
      return fired;
    }

    async void ConnectToServer()
    {
      using var call = _client.Connect(new ConnectRequest
      {
        GameId = _registry.GameService.CurrentGameId,
        PlayerId = _registry.GameService.PlayerId,
        TestMode = true
      });

      while (await call.ResponseStream.MoveNext())
      {
        if (this != null)
        {
          var commands = call.ResponseStream.Current;
          StartCoroutine(_registry.CommandService.HandleCommands(commands));
        }
      }
    }

    IEnumerator HandleActionAsync(GameAction action)
    {
      yield return ApplyOptimisticResponse(action);
      // Introduce simulated server delay
      yield return new WaitForSeconds(Random.Range(0.1f, 1f) + (Random.Range(0f, 1f) < 0.1f ? 1f : 0));

      if (IsClientOnlyAction(action))
      {
        // Client-only action, do not send to server
        _currentlyHandlingAction = false;
        yield break;
      }
      
      // Send to server
      if (_fakeActionResponse)
      {
        yield return _registry.SampleData.FakeActionResponse(action);
      }
      else
      {
        var request = new GameRequest
        {
          Action = action,
          GameId = _registry.GameService.CurrentGameId,
          PlayerId = _registry.GameService.PlayerId
        };

        var task = _client.PerformActionAsync(request).GetAwaiter();
        yield return new WaitUntil(() => task.IsCompleted);
        yield return _registry.CommandService.HandleCommands(task.GetResult());
      }

      _currentlyHandlingAction = false;
    }

    IEnumerator ApplyOptimisticResponse(GameAction action)
    {
      switch (action.ActionCase)
      {
        case GameAction.ActionOneofCase.StandardAction:
          _registry.StaticAssets.PlayButtonSound();
          if (action.StandardAction.Update is { } update)
          {
            yield return _registry.CommandService.HandleCommands(update);
          }

          break;
        case GameAction.ActionOneofCase.TogglePanel:
          _registry.StaticAssets.PlayButtonSound();
          _registry.DocumentService.TogglePanel(action.TogglePanel.Open, action.TogglePanel.PanelAddress);
          break;
        case GameAction.ActionOneofCase.DrawCard:
          _registry.StaticAssets.PlayDrawCardStartSound();
          _registry.ActionDisplayForPlayer(PlayerName.User).SpendActions(1);
          _registry.CardService.DrawOptimisticCard();
          break;
        case GameAction.ActionOneofCase.PlayCard:
          yield return HandlePlayCard(action.PlayCard);
          break;
        case GameAction.ActionOneofCase.GainMana:
          _registry.StaticAssets.PlayAddManaSound();
          _registry.ActionDisplayForPlayer(PlayerName.User).SpendActions(1);
          _registry.ManaDisplayForPlayer(PlayerName.User).GainMana(1);
          break;
        case GameAction.ActionOneofCase.InitiateRaid:
          _registry.ActionDisplayForPlayer(PlayerName.User).SpendActions(1);
          yield return _registry.CommandService.HandleCommands(new GameCommand
          {
            VisitRoom = new VisitRoomCommand
            {
              RoomId = action.InitiateRaid.RoomId,
              Initiator = PlayerName.User,
              VisitType = RoomVisitType.InitiateRaid
            }
          });
          break;
        case GameAction.ActionOneofCase.LevelUpRoom:
          _registry.ActionDisplayForPlayer(PlayerName.User).SpendActions(1);
          yield return _registry.CommandService.HandleCommands(new GameCommand
          {
            VisitRoom = new VisitRoomCommand
            {
              RoomId = action.LevelUpRoom.RoomId,
              Initiator = PlayerName.User,
              VisitType = RoomVisitType.LevelUpRoom
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
      _registry.StaticAssets.PlayWhooshSound();
      var position = Errors.CheckNotNull(card.ReleasePosition, "Card does not have release position");

      if (position.PositionCase == ObjectPosition.PositionOneofCase.Room)
      {
        var room = action.Target.RoomId;
        Errors.CheckArgument(room != RoomIdentifier.Unspecified, "No RoomId target provided!");
        // Move to targeted room
        var newPosition = new ObjectPosition();
        newPosition.MergeFrom(position);
        newPosition.Room.RoomId = room;
        position = newPosition;
      }

      return _registry.ObjectPositionService.MoveGameObject(card, position);
    }

    static bool IsClientOnlyAction(GameAction action) => action.ActionCase switch
    {
      GameAction.ActionOneofCase.None => true,
      GameAction.ActionOneofCase.StandardAction => action.StandardAction.Payload.Length == 0,
      GameAction.ActionOneofCase.TogglePanel => !action.TogglePanel.Open,
      _ => false
    };
  }
}