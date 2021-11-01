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

    public void SetRoomsOnBottom(bool roomsOnBottom)
    {
      _sceneBackground.SetRoomsOnBottom(roomsOnBottom);
    }

    public Room FindRoom(RoomId roomId)
    {
      var result = _rooms.Find(r => r.RoomId == roomId);
      return Errors.CheckNotNull(result);
    }

    public IEnumerator AddAsItem(Displayable card, ObjectPositionItem position, bool animate)
    {
      switch (position.ItemLocation)
      {
        case ItemLocation.Left:
          return _leftItems.AddObject(card, animate);
        case ItemLocation.Right:
          return _rightItems.AddObject(card, animate);
        default:
          Debug.LogError($"Unknown item location: {position.ItemLocation}");
          return _rightItems.AddObject(card, animate);
      }
    }

    public IEnumerator AddToRoom(Displayable card, ObjectPositionRoom position, bool animate)
    {
      return FindRoom(position.RoomId).AddCard(card, position.RoomLocation, position.Index, animate);
    }

    public RoomId? ShowRoomSelectorForMousePosition()
    {
      if (_curentRoomSelector)
      {
        _curentRoomSelector!.SpriteRenderer.enabled = false;
      }

      var ray = _registry.MainCamera.ScreenPointToRay(Input.mousePosition);
      var hits = Physics.RaycastNonAlloc(ray, _raycastHitsTempBuffer, 100);
      RoomId? result = null;

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
  }
}