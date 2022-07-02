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

#nullable enable

using System.Collections;
using System.Linq;
using DG.Tweening;
using Spelldawn.Utils;
using UnityEngine;

namespace Spelldawn.Game
{
  public sealed class LongPressCardBrowser : CardBrowser
  {
    [SerializeField] GameObject _closeButton = null!;
    ObjectDisplay? _display;

    protected override BackgroundOverlay BackgroundOverlay => Registry.LongPressOverlay;

    public IEnumerator BrowseCards(ObjectDisplay display)
    {
      var cards = display.AllObjects.OfType<Card>().Select(c => c.Clone()).Cast<Displayable>().ToList();
      if (cards.Count == 0)
      {
        yield break;
      }      
     
      DestroyAll();
      _display = display;
      yield return AddObjects(cards);
      _closeButton.SetActive(true);
    }

    public void Close()
    {
      var sequence = TweenUtils.Sequence("CloseLongPressBrowser");
      foreach (var item in AllObjects)
      {
        sequence.Insert(0,
          item.transform.DOMove(_display!.transform.position, TweenUtils.MoveAnimationDurationSeconds));
        sequence.Insert(0,
          item.transform.DOLocalRotate(_display!.transform.rotation.eulerAngles,
            TweenUtils.MoveAnimationDurationSeconds));
      }

      sequence.AppendCallback(DestroyAll);
    }
  }
}