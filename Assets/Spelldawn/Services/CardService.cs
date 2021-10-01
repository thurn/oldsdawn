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

using DG.Tweening;
using Spelldawn.Game;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class CardService : MonoBehaviour
  {
    const float CardScale = 1.5f;

    [SerializeField] Registry _registry = null!;
    [SerializeField] Deck _deck = null!;
    [SerializeField] Card _cardPrefab = null!;
    [SerializeField] Transform _cardStagingArea = null!;

    SpriteAddress? _userCardBack;
    SpriteAddress? _opponentCardBack;
    Card? _optimisticCard;

    public void Initialize(CardView? userCard, CardView? opponentCard)
    {
      _userCardBack = userCard?.CardBack;
      _opponentCardBack = opponentCard?.CardBack;
    }

    public void DrawOptimisticCard()
    {
      if (_optimisticCard)
      {
        Destroy(_optimisticCard);
      }

      _optimisticCard = ComponentUtils.Instantiate(_cardPrefab);
      _optimisticCard.Render(_registry, new CardView
      {
        CardBack = _userCardBack
      });
      _optimisticCard.transform.position = _deck.transform.position;
      _optimisticCard.transform.localScale = new Vector3(CardScale, CardScale, 1f);
      var initialMoveTarget = new Vector3(
        _deck.transform.position.x - 4,
        _deck.transform.position.y + 2,
        _deck.transform.position.z - 8);

      DOTween.Sequence()
        .Insert(0,
          _optimisticCard.transform.DOMove(initialMoveTarget, 0.5f).SetEase(Ease.OutCubic))
        .Insert(0.5f, _optimisticCard.transform.DOMove(_cardStagingArea.position, 0.5f).SetEase(Ease.OutCubic))
        .Insert(0, _optimisticCard.transform.DORotateQuaternion(_cardStagingArea.rotation, 1.0f).SetEase(Ease.Linear));
    }
  }
}