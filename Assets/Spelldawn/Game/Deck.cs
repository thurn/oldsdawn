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
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class Deck : ObjectDisplay
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] bool _clickable;

    protected override GameContext DefaultGameContext() => Game.GameContext.Deck;

    protected override Vector3 CalculateObjectPosition(int index, int count) => transform.position;

    protected override Vector3? CalculateObjectRotation(int index, int count) => transform.eulerAngles;

    void OnMouseUpAsButton()
    {
      if (_clickable)
      {
        _registry.ActionService.HandleAction(new GameAction
        {
          DrawCard = new DrawCardAction()
        });
      }
    }
  }
}