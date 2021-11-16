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
    const float CardScale = 1.5f;
    [SerializeField] Registry _registry = null!;
    [SerializeField] Card _cardPrefab = null!;
    Card? _optimisticCard;
    SpriteAddress _userCardBack = null!;
    SpriteAddress _opponentCardBack = null!;

    readonly Dictionary<CardId, Card> _cards = new();

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

    public Card FindCard(CardId cardId)
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
      _optimisticCard.Render(_registry, new CardView { CardBackPlayer = PlayerName.User }, GameContext.Staging);
      _optimisticCard.transform.localScale = new Vector3(CardScale, CardScale, 1f);
      _registry.ObjectPositionService.AnimateFromDeckToStaging(_optimisticCard);
    }

    public IEnumerator HandleCreateCardCommand(CreateCardCommand command)
    {
      Errors.CheckNotNull(command.Card);
      Errors.CheckNotNull(command.Card.CardId);

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
        card.transform.localScale = new Vector3(CardScale, CardScale, 1f);

        StartCoroutine(_registry.ObjectPositionService.ObjectDisplayForPosition(command.Position)
          .AddObject(card, animate: false));

        switch (command.Animation)
        {
          case CardCreationAnimation.UserDeckToStaging:
            _registry.ObjectPositionService.AnimateFromDeckToStaging(card);
            waitForStaging = true;
            break;
        }
      }

      card.Render(
        _registry,
        command.Card,
        GameContext.Staging,
        animate: !command.DisableAnimation);

      _cards[command.Card.CardId] = card;

      if (waitForStaging)
      {
        yield return new WaitUntil(() => card.IsRevealed && card.StagingAnimationComplete);
        yield return new WaitForSeconds(0.5f);
      }
    }

    public Card CreateCard(CardView cardView, GameContext gameContext, bool animate)
    {
      var card = ComponentUtils.Instantiate(_cardPrefab);
      card.transform.localScale = new Vector3(CardScale, CardScale, 1f);
      card.Render(_registry, cardView, gameContext, animate: animate);
      _cards[cardView.CardId] = card;
      return card;
    }

    public IEnumerator HandleUpdateCardCommand(UpdateCardCommand command)
    {
      yield return FindCard(command.Card!.CardId).Render(_registry, command.Card).WaitForCompletion();
    }
  }
}