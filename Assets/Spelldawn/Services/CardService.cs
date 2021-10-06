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
    const float CardScale = 1.5f;

    [SerializeField] Registry _registry = null!;
    [SerializeField] Card _cardPrefab = null!;

    readonly Dictionary<CardId, Card> _cards = new();
    Card? _optimisticCard;
    SpriteAddress? _userCardBack;

    public void Initialize(CardView? userCard, CardView? opponentCard)
    {
      if (userCard != null)
      {
        _userCardBack = userCard.CardBack;
        SetCardBacks(userCard, PlayerName.User);
        SetCardBacks(userCard, PlayerName.Opponent);
      }

      if (opponentCard != null)
      {
        foreach (var spriteRenderer in _registry.DeckForPlayer(PlayerName.Opponent)
          .GetComponentsInChildren<SpriteRenderer>())
        {
          spriteRenderer.sprite = _registry.AssetService.GetSprite(opponentCard.CardBack);
        }
      }
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
      _optimisticCard.transform.localScale = new Vector3(CardScale, CardScale, 1f);
      AnimateFromDeckToStaging(_optimisticCard);
    }

    public IEnumerator CreateCard(CreateCardCommand command)
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
        switch (command.Position)
        {
          case CreateCardPosition.UserDeckToStaging:
            AnimateFromDeckToStaging(card);
            waitForStaging = true;
            break;
          case CreateCardPosition.UserDeck:
            card.transform.position = DeckSpawnPosition(PlayerName.User);
            break;
          case CreateCardPosition.OpponentDeck:
            card.transform.position = DeckSpawnPosition(PlayerName.Opponent);
            break;
          case CreateCardPosition.Staging:
            card.transform.position = _registry.CardStagingArea.position;
            waitForStaging = true;
            break;
        }
      }

      card.Render(_registry, command.Card);
      _cards[command.Card.CardId] = card;

      if (waitForStaging)
      {
        yield return new WaitUntil(() => card.IsRevealed && card.StagingAnimationComplete);
        yield return new WaitForSeconds(0.5f);
      }
    }

    public IEnumerator MoveCard(MoveCardCommand command)
    {
      Errors.CheckState(_cards.ContainsKey(command.CardId), $"Card not found: {command.CardId}");
      var card = _cards[command.CardId];

      switch (command.Zone)
      {
        case GameZone.Hand:
          return _registry.HandForPlayer(command.TargetPlayer).AddCard(card);
        case GameZone.Arena:
          return _registry.ArenaService.AddCard(card);
        case GameZone.Deck:
          return MoveTo(card, _registry.DeckForPlayer(command.TargetPlayer).transform);
        case GameZone.Discard:
          throw new NotImplementedException();
        case GameZone.Banished:
          Destroy(card);
          break;
      }

      return CollectionUtils.Yield();
    }

    IEnumerator<YieldInstruction> MoveTo(Card card, Transform target)
    {
      yield return DOTween.Sequence()
        .Insert(0, card.transform.DOMove(target.position, 0.5f))
        .Insert(0, card.transform.DORotateQuaternion(target.rotation, 0.5f))
        .WaitForCompletion();
    }

    void SetCardBacks(CardView? cardView, PlayerName playerName)
    {
      if (cardView != null)
      {
        foreach (var spriteRenderer in _registry.DeckForPlayer(playerName)
          .GetComponentsInChildren<SpriteRenderer>())
        {
          spriteRenderer.sprite = _registry.AssetService.GetSprite(cardView.CardBack);
        }
      }
    }

    void AnimateFromDeckToStaging(Card card)
    {
      var target = DeckSpawnPosition(PlayerName.User);
      card.transform.position = target;
      var initialMoveTarget = new Vector3(
        target.x - 4,
        target.y + 2,
        target.z - 8);

      DOTween.Sequence()
        .Insert(0,
          card.transform.DOMove(initialMoveTarget, 0.5f).SetEase(Ease.OutCubic))
        .Insert(0.5f, card.transform.DOMove(_registry.CardStagingArea.position, 0.5f).SetEase(Ease.OutCubic))
        .Insert(0, card.transform.DORotateQuaternion(_registry.CardStagingArea.rotation, 1.0f).SetEase(Ease.Linear))
        .AppendCallback(() => card.StagingAnimationComplete = true);
    }

    Vector3 DeckSpawnPosition(PlayerName playerName) =>
      _registry.DeckForPlayer(playerName).transform.position + new Vector3(0f, 1f, 0f);
  }
}