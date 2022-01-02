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

using System.Collections;
using Spelldawn.Protos;
using Spelldawn.Services;
using TMPro;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class IdentityCard : StackObjectDisplay, ArrowService.IArrowDelegate
  {
    [Header("Identity")] [SerializeField] Registry _registry = null!;
    [SerializeField] SpriteRenderer _image = null!;
    [SerializeField] SpriteRenderer _frame = null!;
    [SerializeField] TextMeshPro _scoreText = null!;
    [SerializeField] GameObject _raidSymbol = null!;
    [SerializeField] PlayerName _owner;

    public IdentityAction.IdentityActionOneofCase? DragAction { get; set; }

    public bool RaidSymbolShown
    {
      set => _raidSymbol.SetActive(value);
    }

    protected override Registry Registry => _registry;

    protected override GameContext DefaultGameContext() => GameContext.Identity;

    protected override float? CalculateObjectScale(int index, int count) => 0f;

    public override float DefaultScale => 2.0f;

    public override bool IsContainer() => false;

    public IEnumerator RenderPlayerInfo(PlayerInfo playerInfo)
    {
      _registry.AssetService.AssignSprite(_image, playerInfo.Portrait);
      _registry.AssetService.AssignSprite(_frame, playerInfo.PortraitFrame);
      yield break;
    }

    public IEnumerator RenderScore(ScoreView scoreView)
    {
      _scoreText.text = scoreView.Score.ToString();
      yield break;
    }

    public override bool MouseDown()
    {
      base.MouseDown();

      if (_owner == PlayerName.User && _registry.ActionService.CanInitiateAction())
      {
        _registry.StaticAssets.PlayCardSound();
        switch (DragAction)
        {
          case IdentityAction.IdentityActionOneofCase.InitiateRaid:
            _registry.ArrowService.ShowArrow(ArrowService.Type.Red, transform, this);
            break;
          case IdentityAction.IdentityActionOneofCase.LevelUpRoom:
            _registry.ArrowService.ShowArrow(ArrowService.Type.Green, transform, this);
            break;
        }
      }

      return true;
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
      _registry.ArenaService.ShowRoomSelectorForMousePosition();
    }

    public void OnArrowReleased(Vector3 position)
    {
      if (_registry.ArenaService.CurrentRoomSelector is { } selectedRoom)
      {
        switch (DragAction)
        {
          case IdentityAction.IdentityActionOneofCase.InitiateRaid:
            _registry.ActionService.HandleAction(new GameAction
            {
              InitiateRaid = new InitiateRaidAction
              {
                RoomId = selectedRoom.RoomId
              }
            });
            break;
          case IdentityAction.IdentityActionOneofCase.LevelUpRoom:
            _registry.ActionService.HandleAction(new GameAction
            {
              LevelUpRoom = new LevelUpRoomAction
              {
                RoomId = selectedRoom.RoomId
              }
            });
            break;
        }
      }
      else
      {
        _registry.StaticAssets.PlayCardSound();
      }

      _registry.ArenaService.HideRoomSelector();
    }

    protected override void OnSetGameContext(GameContext _, GameContext gameContext, int? index = null)
    {
    }
  }
}