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
using Spelldawn.Utils;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class Deck : StackObjectDisplay
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] bool _clickable;
    [SerializeField] PlayerName _owner;

    protected override Registry Registry => _registry;

    protected override GameContext DefaultGameContext() => GameContext.Deck;

    public void SetCardBacks(SpriteAddress spriteAddress)
    {
      foreach (var obj in AllObjects)
      {
        var spriteRenderer = obj.GetComponent<SpriteRenderer>();
        if (spriteRenderer)
        {
          // Update card back images for placeholder cards
          spriteRenderer.sprite = _registry.AssetService.GetSprite(spriteAddress);          
        }
      }
    }

    protected override void LongPress()
    {
      StartCoroutine(_registry.LongPressCardBrowser.BrowseCards(this));
    }

    void OnMouseUpAsButton()
    {
      if (_clickable && _registry.ActionService.CanExecuteAction(GameAction.ActionOneofCase.DrawCard))
      {
        _registry.ActionService.HandleAction(new GameAction
        {
          DrawCard = new DrawCardAction()
        });
      }
    }
  }
}