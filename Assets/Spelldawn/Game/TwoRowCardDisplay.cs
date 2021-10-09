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

using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class TwoRowCardDisplay : CardDisplay
  {
    [SerializeField] float _width;
    [SerializeField] float _height;
    [SerializeField] float _fourCardOffset;

    protected override SortingOrder.Type SortingType => SortingOrder.Type.Arena;

    protected override Vector3 CalculateCardPosition(Card card, int index, int count)
    {
      return count switch
      {
        1 => transform.position,
        2 => transform.position + new Vector3(XPosition(index, count), 0, 0),
        _ => transform.position + new Vector3(XPosition(index, count), 0, ZPosition(index, count))
      };
    }

    protected override Vector3? CalculateCardRotation(Card card, int index, int count) =>
      new Vector3(x: 270, y: 0, 0);

    float XPosition(int index, int count) =>
      count < 5
        ? Mathf.Lerp(_width * -_fourCardOffset, _width * _fourCardOffset, XPercentage(index, count))
        : Mathf.Lerp(_width * -0.5f, _width * 0.5f, XPercentage(index, count));

    float ZPosition(int index, int count) =>
      index < count / 2 ? _height / -2f : _height / 2f;

    float XPercentage(int index, int count)
    {
      if (count < 3)
      {
        return index == 0 ? 0 : 1;
      }
      else
      {
        var rowSize = count / 2;
        return index < rowSize ? index / (count - rowSize - 1f) : (index - rowSize) / (count - rowSize - 1f);
      }
    }

    void OnDrawGizmosSelected()
    {
      Gizmos.color = Color.blue;
      Gizmos.DrawSphere(transform.position + new Vector3(_width / 2f, 0, _height / 2f), radius: 1);
      Gizmos.DrawSphere(transform.position + new Vector3(_width / -2f, 0, _height / 2f), radius: 1);
      Gizmos.DrawSphere(transform.position + new Vector3(_width / 2f, 0, _height / -2f), radius: 1);
      Gizmos.DrawSphere(transform.position + new Vector3(_width / -2f, 0, _height / -2f), radius: 1);
    }
  }
}