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
using Spelldawn.Game;
using Spelldawn.Protos;
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class ArenaService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] LinearCardDisplay _leftItems = null!;
    [SerializeField] LinearCardDisplay _rightItems = null!;

    public IEnumerator AddAsItem(Card card, CardPositionItem position, bool animate)
    {
      switch (position.ItemLocation)
      {
        case ItemLocation.Left:
          return _leftItems.AddCard(card, animate);
        case ItemLocation.Right:
          return _rightItems.AddCard(card, animate);
        default:
          Debug.LogError($"Unknown item location: {position.ItemLocation}");
          return _rightItems.AddCard(card, animate);
      }
    }

    public IEnumerator AddToRoom(Card card, CardPositionRoom position, bool animate)
    {
      yield break;
    }
  }
}