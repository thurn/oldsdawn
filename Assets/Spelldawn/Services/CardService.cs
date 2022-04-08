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
using System.Collections;
using System.Collections.Generic;
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
    [SerializeField] Registry _registry = null!;
    [SerializeField] Card _cardPrefab = null!;
    [SerializeField] Card _tokenCardPrefab = null!;
    [SerializeField] ObjectDisplay _infoZoomLeft = null!;
    [SerializeField] ObjectDisplay _infoZoomRight = null!;

    Card? _optimisticCard;
    SpriteAddress _userCardBack = null!;
    SpriteAddress _opponentCardBack = null!;

    readonly Dictionary<CardIdentifier, Card> _cards = new();

    public bool CurrentlyDragging { get; set; }

    public void SetCardBacks(SpriteAddress? userCardBack, SpriteAddress? opponentCardBack)
    {
      if (userCardBack != null)
      {
        _registry.DeckForPlayer(PlayerName.User).SetCardBacks(userCardBack);
        _userCardBack = userCardBack;
      }

      if (opponentCardBack != null)
      {
        _registry.DeckForPlayer(PlayerName.Opponent).SetCardBacks(opponentCardBack);
        _opponentCardBack = opponentCardBack;
      }
    }

    public SpriteAddress GetCardBack(PlayerName playerName) =>
      Errors.CheckNotDefault(playerName) == PlayerName.User ? _userCardBack : _opponentCardBack;

    public Card FindCard(CardIdentifier cardId)
    {
      Errors.CheckState(_cards.ContainsKey(cardId), $"Card Id {cardId} not found");
      return _cards[cardId];
    }

    public void DrawOptimisticCard()
    {
      if (_optimisticCard)
      {
        Destroy(_optimisticCard);
      }

      _optimisticCard = InstantiateCardPrefab(CardPrefab.Standard);
      _optimisticCard.Render(_registry, new CardView { OwningPlayer = PlayerName.User }, GameContext.Staging);
      _optimisticCard.transform.localScale = new Vector3(Card.CardScale, Card.CardScale, 1f);
      _registry.ObjectPositionService.PlayDrawCardAnimation(_optimisticCard);
    }

    public IEnumerator HandleCreateOrUpdateCardCommand(CreateOrUpdateCardCommand command)
    {
      var cardId = Errors.CheckNotNull(command.Card.CardId);

      if (_cards.ContainsKey(cardId))
      {
        yield return FindCard(cardId)
          .Render(_registry, command.Card, animate: !command.DisableFlipAnimation)
          ?.WaitForCompletion();
      }
      else
      {
        yield return HandleCreateCard(command);
      }
    }

    IEnumerator HandleCreateCard(CreateOrUpdateCardCommand command)
    {
      var isOptimistic = (bool)_optimisticCard;
      var waitForStaging = false;
      Card card;
      if (isOptimistic)
      {
        waitForStaging = true;
        card = _optimisticCard!;
        _optimisticCard = null;
      }
      else
      {
        card = InstantiateCardPrefab(command.Card!.Prefab);
        card.transform.localScale = new Vector3(Card.CardScale, Card.CardScale, 1f);

        if (command.CreateAnimation != CardCreationAnimation.DrawCard)
        {
          StartCoroutine(
            _registry.ObjectPositionService.ObjectDisplayForPosition(
                Errors.CheckNotNull(command.CreatePosition, "No create position specified"))
              .AddObject(card, animate: false));
        }
      }

      card.Render(
        _registry,
        command.Card,
        GameContext.Staging,
        animate: !command.DisableFlipAnimation);
      _cards[Errors.CheckNotNull(command.Card.CardId)] = card;

      if (!isOptimistic && command.CreateAnimation == CardCreationAnimation.DrawCard)
      {
        _registry.ObjectPositionService.PlayDrawCardAnimation(card);
        waitForStaging = true;
      }

      if (waitForStaging)
      {
        yield return new WaitUntil(() => card.IsRevealed && card.StagingAnimationComplete);
        yield return new WaitForSeconds(0.5f);
      }
    }

    public Card CreateAndAddCard(CardView cardView, GameContext gameContext, bool animate)
    {
      var card = InstantiateCardPrefab(cardView.Prefab);
      card.transform.localScale = new Vector3(Card.CardScale, Card.CardScale, 1f);
      card.Render(_registry, cardView, gameContext, animate: animate);
      _cards[Errors.CheckNotNull(cardView.CardId)] = card;
      return card;
    }

    public void DisplayInfoZoom(Vector3 worldMousePosition, Card card)
    {
      StartCoroutine(InfoZoom(worldMousePosition, card));
    }

    IEnumerator InfoZoom(Vector3 worldMousePosition, Card card)
    {
      ClearInfoZoom();

      var zoomed = card.Clone();
      zoomed.transform.localScale = new Vector3(Card.CardScale, Card.CardScale, 1f);
      zoomed.SetGameContext(GameContext.InfoZoom);
      zoomed.gameObject.name = $"{card.name} InfoZoom";
      var container = worldMousePosition.x > 0 ? _infoZoomRight : _infoZoomLeft;

      yield return container.AddObject(zoomed, animate: false);

      if (zoomed.SupplementalInfo != null)
      {
        _registry.DocumentService.RenderSupplementalCardInfo(
          zoomed,
          zoomed.SupplementalInfo,
          worldMousePosition.x < 0);
      }
    }

    public void ClearInfoZoom()
    {
      _registry.DocumentService.ClearSupplementalCardInfo();
      _infoZoomLeft.DestroyAll();
      _infoZoomRight.DestroyAll();
    }

    public IEnumerator HandleDestroyCard(CardIdentifier cardId)
    {
      var card = FindCard(cardId);
      _cards.Remove(cardId);
      if (card.Parent && card.Parent != null)
      {
        card.Parent.RemoveObjectIfPresent(card, animate: false);
      }

      Destroy(card.gameObject);
      yield break;
    }

    Card InstantiateCardPrefab(CardPrefab prefab) => ComponentUtils.Instantiate(prefab switch
    {
      CardPrefab.Standard => _cardPrefab,
      CardPrefab.TokenCard => _tokenCardPrefab,
      _ => throw new ArgumentOutOfRangeException()
    });
  }
}