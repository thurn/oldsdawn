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

using Spelldawn.Protos;
using Spelldawn.Services;
using TMPro;
using UnityEngine;
using UnityEngine.Rendering;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class IdentityCard : AbstractCard, ArrowService.IArrowDelegate
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] bool _canDrag;
    [SerializeField] SpriteRenderer _frame = null!;
    [SerializeField] TextMeshPro _scoreText = null!;
    [SerializeField] SortingGroup _sortingGroup = null!;
    [SerializeField] GameObject _raidSymbol = null!;
    RoomId? _selectedRoom;

    public override SortingOrder? SortingOrder
    {
      set => value?.ApplyTo(_sortingGroup);
    }

    public override void SetRenderingMode(RenderingMode renderingMode)
    {
    }

    public bool RaidSymbolShown
    {
      set => _raidSymbol.SetActive(value);
    }

    void OnMouseDown()
    {
      if (_canDrag)
      {
        _registry.ArrowService.ShowArrow(ArrowService.Type.Red, transform, this);
      }
    }

    public void OnArrowMoved(Vector3 position)
    {
      _selectedRoom = _registry.ArenaService.ShowRoomSelectorForMousePosition();
    }

    public void OnArrowReleased(Vector3 position)
    {
      _registry.ArenaService.HideRoomSelector();
      if (_selectedRoom is { } selectedRoom)
      {
        _registry.ActionService.HandleAction(new GameAction
        {
          InitiateRaid = new InitiateRaidAction
          {
            RoomId = selectedRoom
          }
        });
      }
    }
  }
}