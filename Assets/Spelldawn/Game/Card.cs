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
using DG.Tweening;
using Spelldawn.Protos;
using Spelldawn.Services;
using TMPro;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class Card : MonoBehaviour
  {
    [SerializeField] SpriteRenderer _cardBack = null!;
    [SerializeField] Transform _cardFront = null!;
    [SerializeField] SpriteRenderer _image = null!;
    [SerializeField] SpriteRenderer _frame = null!;
    [SerializeField] SpriteRenderer _titleBackground = null!;
    [SerializeField] TextMeshPro _title = null!;
    [SerializeField] TextMeshPro _rulesText = null!;
    [SerializeField] SpriteRenderer _jewel = null!;
    [SerializeField] bool _isRevealed;
    Registry _registry = null!;

    public void Render(Registry registry, CardView cardView)
    {
      _registry = registry;
      _cardBack.sprite = _registry.AssetService.GetSprite(cardView.CardBack);

      if (cardView.RevealedCard != null)
      {
        if (_isRevealed)
        {
          RenderRevealedCard(cardView.RevealedCard);
        }
        else
        {
          Flip(_cardFront, _cardBack, () => RenderRevealedCard(cardView.RevealedCard));
        }
      }
      else
      {
        if (_isRevealed)
        {
          Flip(_cardBack, _cardFront, RenderHiddenCard);
        }
        else
        {
          RenderHiddenCard();
        }
      }
    }

    static void Flip(Component faceUp, Component faceDown, Action onFlipped)
    {
      DOTween.Sequence()
        .Insert(atPosition: 0, faceDown.transform.DOLocalRotate(new Vector3(x: 0, y: 90, z: 0), duration: 0.2f))
        .InsertCallback(atPosition: 0.2f, () =>
        {
          faceUp.gameObject.SetActive(value: true);
          faceUp.transform.localRotation = Quaternion.Euler(x: 0, y: 90, z: 0);
          faceDown.gameObject.SetActive(value: false);
          onFlipped();
        })
        .Insert(atPosition: 0.2f, faceUp.transform.DOLocalRotate(Vector3.zero, duration: 0.3f));
    }

    void RenderRevealedCard(RevealedCardView cardView)
    {
      gameObject.name = cardView.Title.Text;
      _cardBack.gameObject.SetActive(value: false);
      _cardFront.gameObject.SetActive(value: true);
      _image.sprite = _registry.AssetService.GetSprite(cardView.Image);
      _frame.sprite = _registry.AssetService.GetSprite(cardView.CardFrame);
      _titleBackground.sprite = _registry.AssetService.GetSprite(cardView.TitleBackground);
      _title.text = cardView.Title.Text;
      _rulesText.text = cardView.RulesText.Text;
      _jewel.sprite = _registry.AssetService.GetSprite(cardView.Jewel);
    }

    void RenderHiddenCard()
    {
      gameObject.name = "Hidden Card";
      _cardBack.gameObject.SetActive(value: true);
      _cardFront.gameObject.SetActive(value: false);
    }
  }
}