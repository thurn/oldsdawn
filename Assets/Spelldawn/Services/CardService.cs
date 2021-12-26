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

      _optimisticCard = ComponentUtils.Instantiate(_cardPrefab);
      _optimisticCard.Render(_registry, new CardView { OwningPlayer = PlayerName.User }, GameContext.Staging);
      _optimisticCard.transform.localScale = new Vector3(Card.CardScale, Card.CardScale, 1f);
      _registry.ObjectPositionService.PlayDrawCardAnimation(_optimisticCard);
    }

    public IEnumerator HandleCreateOrUpdateCardCommand(CreateOrUpdateCardCommand command)
    {
      var cardId = Errors.CheckNotNull(command.Card.CardId);

      if (_cards.ContainsKey(cardId))
      {
        yield return FindCard(command.Card!.CardId)
          .Render(_registry, command.Card, animate: !command.DisableFlipAnimation)
          .WaitForCompletion();
      }
      else
      {
        yield return HandleCreateCard(command);
      }
    }

    IEnumerator HandleCreateCard(CreateOrUpdateCardCommand command)
    {
      var waitForStaging = false;
      Card card;
      if (_optimisticCard)
      {
        waitForStaging = true;
        card = _optimisticCard!;
        _optimisticCard = null;
      }
      else
      {
        card = ComponentUtils.Instantiate(_cardPrefab);
        card.transform.localScale = new Vector3(Card.CardScale, Card.CardScale, 1f);

        StartCoroutine(_registry.ObjectPositionService.ObjectDisplayForPosition(command.CreatePosition)
          .AddObject(card, animate: false));

        switch (command.CreateAnimation)
        {
          case CardCreationAnimation.DrawCard:
            _registry.ObjectPositionService.PlayDrawCardAnimation(card);
            waitForStaging = true;
            break;
        }
      }

      card.Render(
        _registry,
        command.Card,
        GameContext.Staging,
        animate: !command.DisableFlipAnimation);

      _cards[Errors.CheckNotNull(command.Card.CardId)] = card;

      if (waitForStaging)
      {
        yield return new WaitUntil(() => card.IsRevealed && card.StagingAnimationComplete);
        yield return new WaitForSeconds(0.5f);
      }
    }

    public Card CreateAndAddCard(CardView cardView, GameContext gameContext, bool animate)
    {
      var card = ComponentUtils.Instantiate(_cardPrefab);
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

      var cardView = Errors.CheckNotNull(card.CardView);
      var revealed = Errors.CheckNotNull(cardView.RevealedCard);
      var zoomed = ComponentUtils.Instantiate(_cardPrefab);
      zoomed.transform.localScale = new Vector3(Card.CardScale, Card.CardScale, 1f);
      zoomed.Render(_registry, card.CardView!, GameContext.InfoZoom, animate: false);
      zoomed.gameObject.name = $"{card.name} InfoZoom";
      var container = worldMousePosition.x > 0 ? _infoZoomRight : _infoZoomLeft;

      yield return container.AddObject(zoomed, animate: false);

      if (revealed.SupplementalInfo != null)
      {
        _registry.DocumentService.RenderSupplementalCardInfo(
          zoomed,
          revealed.SupplementalInfo,
          worldMousePosition.x > 0 ? CardNodeAnchorPosition.Left : CardNodeAnchorPosition.Right);
      }
    }

    public void ClearInfoZoom()
    {
      _registry.DocumentService.ClearCardControls();
      _infoZoomLeft.DestroyAll();
      _infoZoomRight.DestroyAll();
    }
  }
}