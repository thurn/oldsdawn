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
using DG.Tweening;
using Spelldawn.Game;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class ArenaService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] LinearObjectDisplay _leftItems = null!;
    [SerializeField] LinearObjectDisplay _rightItems = null!;
    [SerializeField] List<Room> _rooms = null!;
    [SerializeField] SceneBackground _sceneBackground = null!;
    [SerializeField] TimedEffect _levelUpRoomPrefab = null!;
    [SerializeField] Room? _curentRoomSelector;

    public ObjectDisplay LeftItems => _leftItems;
    public ObjectDisplay RightIems => _rightItems;

    readonly RaycastHit[] _raycastHitsTempBuffer = new RaycastHit[8];

    public IEnumerator RenderArenaView(ArenaView arenaView)
    {
      if (arenaView.RoomsAtBottom is { } b)
      {
        _sceneBackground.SetRoomsOnBottom(b);
      }

      yield break;
    }

    public Room FindRoom(RoomIdentifier roomId)
    {
      var result = _rooms.Find(r => r.RoomId == roomId);
      return Errors.CheckNotNull(result);
    }

    public ObjectDisplay ObjectDisplayForLocation(ClientItemLocation location) => location switch
    {
      ClientItemLocation.Left => _leftItems,
      ClientItemLocation.Right => _rightItems,
      _ => throw new ArgumentOutOfRangeException(nameof(location), location, null)
    };

    public IEnumerator AddAsItem(Displayable card, ObjectPositionItem position, bool animate)
    {
      switch (position.ItemLocation)
      {
        case ClientItemLocation.Left:
          return _leftItems.AddObject(card, animate);
        case ClientItemLocation.Right:
          return _rightItems.AddObject(card, animate);
        default:
          Debug.LogError($"Unknown item location: {position.ItemLocation}");
          return _rightItems.AddObject(card, animate);
      }
    }

    public RoomIdentifier? ShowRoomSelectorForMousePosition()
    {
      if (_curentRoomSelector)
      {
        _curentRoomSelector!.SpriteRenderer.enabled = false;
      }

      var ray = _registry.MainCamera.ScreenPointToRay(Input.mousePosition);
      var hits = Physics.RaycastNonAlloc(ray, _raycastHitsTempBuffer, 100);
      RoomIdentifier? result = null;

      for (var i = 0; i < hits; ++i)
      {
        var hit = _raycastHitsTempBuffer[i];
        var selector = hit.collider.GetComponent<Room>();
        if (selector)
        {
          result = selector.RoomId;
          selector.SpriteRenderer.enabled = true;
          _curentRoomSelector = selector;
          break;
        }
      }

      Array.Clear(_raycastHitsTempBuffer, 0, _raycastHitsTempBuffer.Length);
      return result;
    }

    public void HideRoomSelector()
    {
      if (_curentRoomSelector)
      {
        _curentRoomSelector!.SpriteRenderer.enabled = false;
      }
    }

    public IEnumerator HandleLevelUpRoom(LevelUpRoomCommand command)
    {
      var room = FindRoom(command.RoomId).transform;
      yield return TweenUtils.Sequence("RoomVisit")
        .Append(_registry.IdentityCardForPlayer(command.Initiator).transform
          .DOMove(room.position, 0.3f).SetEase(Ease.OutSine))
        .AppendCallback(() =>
        {
          var effect = _registry.AssetPoolService.Create(_levelUpRoomPrefab, room.position);
          effect.transform.localScale = 5f * Vector3.one;
        })
        .WaitForCompletion();
    }

    public IEnumerator HandleSetGameObjectsEnabled(SetGameObjectsEnabledCommand command)
    {
      foreach (var room in _rooms)
      {
        SetObjectDisplayActive(room.FrontCards, command);
        SetObjectDisplayActive(room.BackCards, command);
      }

      SetObjectDisplayActive(_leftItems, command);
      SetObjectDisplayActive(_rightItems, command);

      SetGameObjectsEnabledForPlayer(PlayerName.User, command);
      SetGameObjectsEnabledForPlayer(PlayerName.Opponent, command);
      yield break;
    }

    void SetGameObjectsEnabledForPlayer(PlayerName playerName, SetGameObjectsEnabledCommand command)
    {
      _registry.ActionDisplayForPlayer(playerName).gameObject.SetActive(command.GameObjectsEnabled);
      _registry.ManaDisplayForPlayer(playerName).gameObject.SetActive(command.GameObjectsEnabled);
      _registry.IdentityCardForPlayer(playerName).gameObject.SetActive(command.GameObjectsEnabled);
      SetObjectDisplayActive(_registry.DeckForPlayer(playerName), command);
      SetObjectDisplayActive(_registry.DiscardPileForPlayer(playerName), command);
      SetObjectDisplayActive(_registry.IdentityCardForPlayer(playerName), command);
      SetObjectDisplayActive(_registry.HandForPlayer(playerName), command);
    }

    void SetObjectDisplayActive(ObjectDisplay objectDisplay, SetGameObjectsEnabledCommand command)
    {
      foreach (var child in objectDisplay.AllObjects)
      {
        child.gameObject.SetActive(command.GameObjectsEnabled);
      }

      objectDisplay.gameObject.SetActive(command.GameObjectsEnabled);
    }
  }
}