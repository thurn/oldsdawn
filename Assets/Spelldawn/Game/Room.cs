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
using System.Collections.Generic;
using Spelldawn.Protos;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class Room : MonoBehaviour
  {
    [SerializeField] RectangularObjectDisplay _inRoom = null!;

    public IEnumerable<Displayable> CardsInRoom => _inRoom.AllObjects;

    // Defenders are sorted in order, index 0 represents the rearmost defender
    [SerializeField] RectangularObjectDisplay _defenders = null!;

    public IEnumerable<Displayable> Defenders => _defenders.AllObjects;

    [SerializeField] RoomId _roomId;
    public RoomId RoomId => _roomId;

    [SerializeField] SpriteRenderer _spriteRenderer = null!;
    public SpriteRenderer SpriteRenderer => _spriteRenderer;

    public IEnumerator AddCard(Displayable card, RoomLocation location, int? index, bool animate) => location switch
    {
      RoomLocation.InRoom => _inRoom.AddObject(card, animate, index),
      _ => _defenders.AddObject(card, animate, index)
    };
  }
}