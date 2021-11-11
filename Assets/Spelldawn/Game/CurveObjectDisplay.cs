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
  public sealed class CurveObjectDisplay : ObjectDisplay
  {
    [SerializeField] int _zRotationMultiplier;
    [SerializeField] GameContext _sortingGameContext;
    [SerializeField] Transform _controlPoint1 = null!;
    [SerializeField] Transform _controlPoint2 = null!;
    [SerializeField] Transform _controlPoint3 = null!;
    [SerializeField] Transform _controlPoint4 = null!;
    [SerializeField] float _gizmoRadius = 1.0f;
    [SerializeField] float _objectScale;
    [SerializeField] float _animationDuration = 0.3f;

    protected override GameContext DefaultGameContext() => _sortingGameContext;

    protected override float AnimationDuration => _animationDuration;

    protected override Vector3 CalculateObjectPosition(int index, int count)
    {
      var curvePosition = CalculateCurvePosition(index, count);
      return CalculateBezierPosition(curvePosition);
    }

    protected override Vector3? CalculateObjectRotation(int index, int count)
    {
      var curvePosition = CalculateCurvePosition(index, count);
      return new Vector3(x: 280, y: 0, _zRotationMultiplier * CalculateZRotation(curvePosition));
    }

    protected override float? CalculateObjectScale(int index, int count) => _objectScale == 0.0 ? null : _objectScale;

    float CalculateCurvePosition(int cardIndex, int cardCount)
    {
      if (cardIndex < 0 || cardIndex >= cardCount)
      {
        throw new ArgumentException("Index out of bounds");
      }

      switch (cardCount)
      {
        case 1:
          return 0.5f;
        case 2:
          return PositionWithinRange(start: 0.4f, end: 0.6f, cardIndex, cardCount);
        case 3:
          return PositionWithinRange(start: 0.3f, end: 0.7f, cardIndex, cardCount);
        case 4:
          return PositionWithinRange(start: 0.2f, end: 0.8f, cardIndex, cardCount);
        case 5:
          return PositionWithinRange(start: 0.1f, end: 0.9f, cardIndex, cardCount);
        default:
          return PositionWithinRange(start: 0.0f, end: 1.0f, cardIndex, cardCount);
      }
    }

    // Given a start,end range on the 0,1 line, returns the position within that range where card 'index' of of
    // 'count' total cards should be positioned
    float PositionWithinRange(float start, float end, int index, int count) =>
      start + index * ((end - start) / (count - 1.0f));

    // Card rotation ranges from 5 to -5
    float CalculateZRotation(float t) => -10.0f * t + 5.0f;

    Vector3 CalculateBezierPosition(float t) =>
      Mathf.Pow(1 - t, p: 3) * _controlPoint1.position +
      3 * Mathf.Pow(1 - t, p: 2) * t * _controlPoint2.position +
      3 * (1 - t) * Mathf.Pow(t, p: 2) * _controlPoint3.position +
      Mathf.Pow(t, p: 3) * _controlPoint4.position;

    void OnDrawGizmosSelected()
    {
      for (var t = 0.0f; t <= 1; t += 0.05f)
      {
        var position = CalculateBezierPosition(t);
        Gizmos.DrawSphere(position, radius: _gizmoRadius);
      }

      Gizmos.color = Color.green;
      Gizmos.DrawSphere(_controlPoint1.position, radius: _gizmoRadius);
      Gizmos.DrawSphere(_controlPoint2.position, radius: _gizmoRadius);
      Gizmos.DrawSphere(_controlPoint3.position, radius: _gizmoRadius);
      Gizmos.DrawSphere(_controlPoint4.position, radius: _gizmoRadius);
    }
  }
}