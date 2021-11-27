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
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class CardBrowser : CurveObjectDisplay
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] bool _active;

    static readonly ObjectPosition BrowserPosition = new()
    {
      Browser = new ObjectPositionBrowser()
    };

    protected override void OnUpdated()
    {
      if (ObjectCount > 0)
      {
        _registry.BackgroundOverlay.Enable(GameContext.Interface, translucent: true);
        _active = true;
      }
      else if (_active)
      {
        _registry.BackgroundOverlay.Disable();
        _active = false;
      }
    }


    public IEnumerator BrowseCards(ObjectPosition atPosition)
    {
      if (_registry.ActionService.CanInitiateAction())
      {
        _registry.ArrowService.HideArrows();

        yield return _registry.ObjectPositionService.HandleMoveGameObjectsAtPosition(
          new MoveGameObjectsAtPositionCommand
          {
            SourcePosition = atPosition,
            TargetPosition = BrowserPosition,
          });

        yield return _registry.DocumentService.RenderMainControls(
          DocumentService.ControlGroup(DocumentService.Button("Close", new GameCommand
          {
            MoveObjectsAtPosition = new MoveGameObjectsAtPositionCommand
            {
              SourcePosition = BrowserPosition,
              TargetPosition = atPosition
            }
          })));
      }
    }
  }
}