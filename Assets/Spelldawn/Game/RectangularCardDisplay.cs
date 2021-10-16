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
  public sealed class RectangularCardDisplay : CardDisplay
  {
    [SerializeField] float _width;
    [SerializeField] float _height;
    [SerializeField] float _initialSpacing;
    [SerializeField] float _cardSize;

    protected override SortingOrder.Type SortingType => SortingOrder.Type.Arena;

    protected override Vector3 CalculateCardPosition(Card card, int index, int count) =>
      transform.position + new Vector3(
        CalculateXOffset(index, count),
        0,
        CalculateZOffset(index, count));

    protected override Vector3? CalculateCardRotation(Card card, int index, int count) =>
      new Vector3(x: 270, y: 0, 0);

    float CalculateXOffset(int index, int count)
    {
      if (count < 3 || count == 3 && index == 2)
      {
        return 0;
      }

      count = (int) Mathf.Ceil(count / 2f);
      if (index >= count)
      {
        index -= count;
      }

      return LinearCardDisplay.CalculateXOffset(_width, _initialSpacing, _cardSize, index, count);
    }

    float CalculateZOffset(int index, int count)
    {
      var availableHeight = Mathf.Min(_height, (_cardSize + _initialSpacing) * count);
      var offset = availableHeight / 2f - _cardSize / 2f;

      if (transform.eulerAngles.y > 90f)
      {
        // Flip the 'front' of the display if we're rotated in world space
        offset = -offset;
      }

      return count switch
      {
        1 => 0,
        _ when index >= (int) Mathf.Ceil(count / 2f) => offset,
        _ => -offset
      };
    }

    void OnDrawGizmosSelected()
    {
      Gizmos.color = Color.blue;
      Gizmos.DrawSphere(transform.position + new Vector3(_width / 2f, 0, _height / 2f), radius: 0.5f);
      Gizmos.DrawSphere(transform.position + new Vector3(_width / 2f, 0, _height / -2f), radius: 0.5f);
      Gizmos.DrawSphere(transform.position, radius: 0.5f);
      Gizmos.DrawSphere(transform.position + new Vector3(_width / -2f, 0, _height / 2f), radius: 0.5f);
      Gizmos.DrawSphere(transform.position + new Vector3(_width / -2f, 0, _height / -2f), radius: 0.5f);
    }
  }
}