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
using UnityEngine.Rendering;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class SortingOrder
  {
    public static SortingOrder Create(Type type, int index = 0) => new(type, index);

    public enum Type
    {
      Hand,
      Staging,
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
      group.sortingOrder = _index + _type switch
      {
        Type.Hand => 100,
        Type.Staging => 200,
        Type.Dragging => 200,
        _ => throw new ArgumentOutOfRangeException()
      };
    }
  }
}