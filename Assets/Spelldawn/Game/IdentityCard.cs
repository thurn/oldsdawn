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

#nullable enable

namespace Spelldawn.Game
{
  public sealed class IdentityCard : StackObjectDisplay, ArrowService.IArrowDelegate
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] bool _canDrag;
    [SerializeField] TextMeshPro _scoreText = null!;
    [SerializeField] GameObject _raidSymbol = null!;

    RoomId? _selectedRoom;

    public IdentityAction? IdentityAction { private get; set; }

    public bool RaidSymbolShown
    {
      set => _raidSymbol.SetActive(value);
    }

    protected override GameContext DefaultGameContext() => GameContext.Arena;

    protected override float? CalculateObjectScale(int index, int count) => 0.1f;

    public void SetScore(int? value)
    {
      if (value is { } v)
      {
        _scoreText.text = v.ToString();
      }
    }

    void OnMouseDown()
    {
      if (_canDrag)
      {
        switch (IdentityAction)
        {
          case Protos.IdentityAction.InitiateRaid:
            _registry.ArrowService.ShowArrow(ArrowService.Type.Red, transform, this);
            break;
          case Protos.IdentityAction.LevelUpRoom:
            _registry.ArrowService.ShowArrow(ArrowService.Type.Green, transform, this);
            break;
        }
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
        switch (IdentityAction)
        {
          case Protos.IdentityAction.InitiateRaid:
            _registry.ActionService.HandleAction(new GameAction
            {
              InitiateRaid = new InitiateRaidAction
              {
                RoomId = selectedRoom
              }
            });
            break;
          case Protos.IdentityAction.LevelUpRoom:
            _registry.ActionService.HandleAction(new GameAction
            {
              LevelUpRoom = new LevelUpRoomAction
              {
                RoomId = selectedRoom
              }
            });
            break;
        }
      }
    }

    protected override void OnSetGameContext(GameContext _, GameContext gameContext, int? index = null)
    {
    }
  }
}