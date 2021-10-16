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
  public abstract class CardDisplay : MonoBehaviour
  {
    protected List<AbstractCard> Cards { get; } = new();

    public List<AbstractCard> AllCards => new(Cards);

    public IEnumerator<YieldInstruction> AddCard(AbstractCard card, bool animate = true, int? index = null)
    {
      card.Parent = this;

      if (!Cards.Contains(card))
      {
        if (index is { } i)
        {
          Cards.Insert(i, card);
        }
        else
        {
          Cards.Add(card);
        }
      }

      return MoveCardsToPosition(animate);
    }

    public int RemoveCard(AbstractCard card, bool animate = true)
    {
      var index = Cards.FindIndex(c => c == card);
      Errors.CheckNonNegative(index);
      Cards.RemoveAt(index);
      StartCoroutine(MoveCardsToPosition(animate));
      return index;
    }

    public void RemoveCardIfPresent(AbstractCard card, bool animate = true)
    {
      Cards.Remove(card);
      StartCoroutine(MoveCardsToPosition(animate));
    }

    protected abstract SortingOrder.Type SortingType { get; }

    protected virtual float AnimationDuration => 0.3f;

    protected abstract Vector3 CalculateCardPosition(int index, int count);

    protected virtual Vector3? CalculateCardRotation(int index, int count) => null;

    IEnumerator<YieldInstruction> MoveCardsToPosition(bool animate)
    {
      var sequence = DOTween.Sequence();
      for (var i = 0; i < Cards.Count; ++i)
      {
        var card = Cards[i];
        var position = CalculateCardPosition(i, Cards.Count);
        var rotation = CalculateCardRotation(i, Cards.Count);

        if (animate)
        {
          sequence.Insert(atPosition: 0, card.transform.DOMove(position, duration: AnimationDuration));
        }
        else
        {
          card.transform.position = position;
        }

        if (rotation is { } vector)
        {
          if (animate)
          {
            sequence.Insert(atPosition: 0, card.transform.DOLocalRotate(vector, duration: AnimationDuration));
          }
          else
          {
            card.transform.localEulerAngles = vector;
          }
        }

        card.SortingOrder = SortingOrder.Create(SortingType, 10 + i);
      }

      if (animate)
      {
        yield return sequence.WaitForCompletion();
      }
    }
  }
}