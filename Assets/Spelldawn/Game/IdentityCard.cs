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
    [Header("Identity Card")] [SerializeField]
    Registry _registry = null!;

    [SerializeField] TextMeshPro _scoreText = null!;
    [SerializeField] GameObject _raidSymbol = null!;
    [SerializeField] PlayerName _owner;

    IdentityAction? _identityAction;
    Card? _identityCard;

    RoomId? _selectedRoom;

    public bool RaidSymbolShown
    {
      set => _raidSymbol.SetActive(value);
    }

    protected override GameContext DefaultGameContext() => GameContext.Identity;

    protected override float? CalculateObjectScale(int index, int count) => 0f;

    public override float DefaultScale => 2.0f;

    public override bool IsContainer() => false;

    public void Render(IdentityAction? identityAction, PlayerView? view)
    {
      _identityAction = identityAction;

      if (view?.PlayerInfo?.IdentityCard is { } ic)
      {
        if (_identityCard)
        {
          _identityCard!.Render(_registry, ic, GameContext.Identity, animate: false);
        }
        else
        {
          _identityCard = _registry.CardService.CreateCard(ic, GameContext.Identity, animate: false);
          StartCoroutine(AddObject(_identityCard, animate: false));
        }
      }

      if (view?.Score?.Score is { } s)
      {
        _scoreText.text = s.ToString();
      }
    }

    protected override void RunMouseDown()
    {
      if (_owner == PlayerName.User && _registry.ActionService.CanInitiateAction())
      {
        switch (_identityAction)
        {
          case IdentityAction.InitiateRaid:
            _registry.ArrowService.ShowArrow(ArrowService.Type.Red, transform, this);
            break;
          case IdentityAction.LevelUpRoom:
            _registry.ArrowService.ShowArrow(ArrowService.Type.Green, transform, this);
            break;
        }
      }
    }

    protected override void LongPress()
    {
      StartCoroutine(_registry.CardBrowser.BrowseCards(new ObjectPosition
      {
        Identity = new ObjectPositionIdentity
        {
          Owner = _owner
        }
      }));
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
        switch (_identityAction)
        {
          case IdentityAction.InitiateRaid:
            _registry.ActionService.HandleAction(new GameAction
            {
              InitiateRaid = new InitiateRaidAction
              {
                RoomId = selectedRoom
              }
            });
            break;
          case IdentityAction.LevelUpRoom:
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