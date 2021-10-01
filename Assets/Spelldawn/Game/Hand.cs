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
using System.Collections.Generic;
using DG.Tweening;
using Spelldawn.Protos;
using Spelldawn.Services;
using Spelldawn.Utils;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class Hand : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] Transform _deckPosition = null!;
    [SerializeField] float _initialCardScale;
    [SerializeField] float _finalCardScale;
    [SerializeField] int _zRotationMultiplier;
    [SerializeField] Transform _controlPoint1 = null!;
    [SerializeField] Transform _controlPoint2 = null!;
    [SerializeField] Transform _controlPoint3 = null!;
    [SerializeField] Transform _controlPoint4 = null!;
    [SerializeField] Card _cardPrefab = null!;

    readonly List<Card> _cards = new();

    public void DrawCard(CardView cardView)
    {
      var card = ComponentUtils.Instantiate(_cardPrefab);
      card.Render(_registry, cardView);
      card.transform.position = _deckPosition.position;
      card.transform.localScale = Vector2.one * _initialCardScale;
      card.transform.SetParent(transform);
      _cards.Add(card);
      AnimateCardsToPosition();
    }

    void AnimateCardsToPosition(Action? onComplete = null)
    {
      var sequence = DOTween.Sequence();
      for (var i = 0; i < _cards.Count; ++i)
      {
        var card = _cards[i];
        var curvePosition = CalculateCurvePosition(i);
        var t = card.transform;
        t.SetSiblingIndex(i);
        sequence.Insert(atPosition: 0, t.DOScale(_finalCardScale, duration: 0.3f));
        sequence.Insert(atPosition: 0,
          t.DOMove(CalculateBezierPosition(curvePosition), duration: 0.3f));
        sequence.Insert(atPosition: 0,
          t.DOLocalRotate(new Vector3(x: 0, y: 0,
            _zRotationMultiplier * CalculateZRotation(curvePosition)), duration: 0.3f));
      }

      sequence.AppendCallback(() => onComplete?.Invoke());
    }

    float CalculateCurvePosition(int cardIndex)
    {
      if (cardIndex < 0 || cardIndex >= _cards.Count)
      {
        throw new ArgumentException("Index out of bounds");
      }

      switch (_cards.Count)
      {
        case 1:
          return 0.5f;
        case 2:
          return PositionWithinRange(start: 0.4f, end: 0.6f, cardIndex, _cards.Count);
        case 3:
          return PositionWithinRange(start: 0.3f, end: 0.7f, cardIndex, _cards.Count);
        case 4:
          return PositionWithinRange(start: 0.2f, end: 0.8f, cardIndex, _cards.Count);
        case 5:
          return PositionWithinRange(start: 0.15f, end: 0.85f, cardIndex, _cards.Count);
        case 6:
          return PositionWithinRange(start: 0.1f, end: 0.9f, cardIndex, _cards.Count);
        default:
          return PositionWithinRange(start: 0.0f, end: 1.0f, cardIndex, _cards.Count);
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
        Gizmos.DrawSphere(position, radius: 1);
      }

      Gizmos.color = Color.green;
      Gizmos.DrawSphere(_controlPoint1.position, radius: 1);
      Gizmos.DrawSphere(_controlPoint2.position, radius: 1);
      Gizmos.DrawSphere(_controlPoint3.position, radius: 1);
      Gizmos.DrawSphere(_controlPoint4.position, radius: 1);
    }
  }
}