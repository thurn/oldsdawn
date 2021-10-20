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
  public sealed class RaidParticipantsCardDisplay : CardDisplay
  {
    [SerializeField] float _width;
    [SerializeField] float _initialSpacing;
    [SerializeField] float _cardSize;
    [SerializeField] float _rotation = 270;

    protected override SortingOrder.Type DefaultSortingType() => SortingOrder.Type.Raid;

    protected override Vector3 CalculateCardPosition(int index, int count) =>
      transform.position + new Vector3(LinearCardDisplay.CalculateXOffset(
        _width, _initialSpacing, _cardSize, index, count,
        minOffsetMultiplier: 0f,
        maxOffsetMultiplier: 1f), 0, 0);

    protected override Vector3? CalculateCardRotation(int index, int count) =>
      new Vector3(x: _rotation, y: 0, 0);

    void OnDrawGizmosSelected()
    {
      Gizmos.color = Color.red;
      Gizmos.DrawSphere(transform.position + new Vector3(_width / 2f, 0, 0), radius: 1);
      Gizmos.DrawSphere(transform.position, radius: 1);
    }
  }
}