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
using UnityEngine;
using UnityEngine.Rendering;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class SortingOrder
  {
    public static SortingOrder Create(Type type, int index = 0) => new(type, index);

    public enum Type
    {
      Arena,
      Deck,
      Discard,
      Raid,
      Hand,
      Interface,
      Staging,
      Effects,
      Dragging
    }

    readonly Type _type;
    readonly int _index;

    SortingOrder(Type type, int index)
    {
      _type = type;
      _index = index;
    }

    public void ApplyTo(SortingGroup group)
    {
      group.sortingOrder = Position();
    }

    public void ApplyTo(Renderer renderer)
    {
      renderer.sortingOrder = Position();
    }

    int Position() => _index + _type switch
    {
      Type.Arena => 100,
      Type.Deck => 200,
      Type.Discard => 300,
      Type.Raid => 400,
      Type.Hand => 500,
      Type.Interface => 600,
      Type.Staging => 700,
      Type.Effects => 800,
      Type.Dragging => 900,
      _ => throw new ArgumentOutOfRangeException()
    };
  }

  public sealed class ApplySortingOrder : MonoBehaviour
  {
    [SerializeField] SortingOrder.Type _type;

    void Start()
    {
      var group = gameObject.AddComponent<SortingGroup>();
      SortingOrder.Create(_type).ApplyTo(group);
    }
  }
}