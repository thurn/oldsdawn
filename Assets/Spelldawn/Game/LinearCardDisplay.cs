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

#nullable enable

namespace Spelldawn.Game
{
  public sealed class LinearCardDisplay : CardDisplay
  {
    [SerializeField] float _width;
    [SerializeField] float _initialSpacing;

    protected override SortingOrder.Type SortingType => SortingOrder.Type.Arena;

    protected override Vector3 CalculateCardPosition(Card card, int index, int count)
    {
      var imageWidth = card.ImageWidth;
      var availableWidth = Mathf.Min(_width, (imageWidth + _initialSpacing) * count);
      var offset = (availableWidth / 2f - imageWidth / 2f);
      var minX = -offset;
      var maxX = offset;

      return count switch
      {
        1 => transform.position,
        _ => transform.position + new Vector3(Mathf.Lerp(minX, maxX, index / (count - 1f)), 0, 0)
      };
    }

    protected override Vector3? CalculateCardRotation(Card card, int index, int count) =>
      new Vector3(x: 270, y: 0, 0);

    void OnDrawGizmos()
    {
      Gizmos.color = Color.blue;
      Gizmos.DrawSphere(transform.position + new Vector3(_width / 2f, 0, 0), radius: 1);
      Gizmos.DrawSphere(transform.position, radius: 1);
      Gizmos.DrawSphere(transform.position + new Vector3(_width / -2f, 0, 0), radius: 1);
    }
  }
}