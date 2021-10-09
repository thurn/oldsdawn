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

using System.Collections.Generic;
using DG.Tweening;
using Spelldawn.Utils;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public abstract class CardList : MonoBehaviour
  {
    protected List<Card> Cards { get; } = new();

    public IEnumerator<YieldInstruction> AddCard(Card card, int? index = null)
    {
      card.Parent = this;
      if (index is { } i)
      {
        Cards.Insert(i, card);
      }
      else
      {
        Cards.Add(card);
      }

      return AnimateCardsToPosition();
    }

    public int RemoveCard(Card card)
    {
      var index = Cards.FindIndex(c => c == card);
      Errors.CheckNonNegative(index);
      Cards.RemoveAt(index);
      StartCoroutine(AnimateCardsToPosition());
      return index;
    }

    protected abstract SortingOrder.Type SortingType { get; }

    protected virtual float AnimationDuration => 0.3f;

    protected abstract Vector3 CalculateCardPosition(Card card, int index, int count);

    protected virtual Vector3? CalculateCardRotation(Card card, int index, int count) => null;

    IEnumerator<YieldInstruction> AnimateCardsToPosition()
    {
      var sequence = DOTween.Sequence();
      for (var i = 0; i < Cards.Count; ++i)
      {
        var card = Cards[i];
        sequence.Insert(atPosition: 0,
          card.transform.DOMove(CalculateCardPosition(card, i, Cards.Count), duration: AnimationDuration));
        if (CalculateCardRotation(card, i, Cards.Count) is { } vector)
        {
          sequence.Insert(atPosition: 0, card.transform.DOLocalRotate(vector, duration: AnimationDuration));
        }

        card.SortingOrder = SortingOrder.Create(SortingType, i);
      }

      yield return sequence.WaitForCompletion();
    }
  }
}