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

    readonly Dictionary<CardId, AbstractCard> _cards = new();

    static readonly CardId UserCardId = new CardId
    {
      IdentityCard = PlayerName.User
    };
    static readonly CardId OpponentCardId = new CardId
    {
      IdentityCard = PlayerName.Opponent
    };

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

      _cards[UserCardId] = _registry.IdentityCardForPlayer(PlayerName.User);
      _cards[OpponentCardId] = _registry.IdentityCardForPlayer(PlayerName.Opponent);
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

    public IEnumerator HandleCreateCardCommand(CreateCardCommand command)
    {
      Errors.CheckNotNull(command.Card);
      Errors.CheckNotNull(command.Card.CardId);

      var waitForStaging = false;
      Card card;
      if (!command.DisallowOptimistic && _optimisticCard)
      {
        waitForStaging = true;
        card = _optimisticCard!;
        _optimisticCard = null;
      }
      else
      {
        card = ComponentUtils.Instantiate(_cardPrefab);
        card.transform.localScale = new Vector3(CardScale, CardScale, 1f);
        StartCoroutine(MoveCardInternal(card, command.Card.OnCreatePosition, animate: false));

        switch (command.Animation)
        {
          case CardCreationAnimation.UserDeckToStaging:
            AnimateFromDeckToStaging(card);
            waitForStaging = true;
            break;
        }
      }

      card.Render(_registry, command.Card, animate: waitForStaging);
      _cards[command.Card.CardId] = card;

      if (waitForStaging)
      {
        yield return new WaitUntil(() => card.IsRevealed && card.StagingAnimationComplete);
        yield return new WaitForSeconds(0.5f);
      }
    }

    public IEnumerator HandleMoveCardCommand(MoveCardCommand command)
    {
      Errors.CheckState(_cards.ContainsKey(command.CardId), $"Card not found: {command.CardId}");
      var card = _cards[command.CardId];
      if (card.Parent)
      {
        card.Parent!.RemoveCardIfPresent(card, !command.DisableAnimation);
      }
      return MoveCardInternal(card, command.Position, !command.DisableAnimation);
    }

    public IEnumerator MoveCard(AbstractCard card, CardPosition targetPosition, bool animate = true)
    {
      if (card.Parent)
      {
        card.Parent!.RemoveCardIfPresent(card, animate);
      }
      return MoveCardInternal(card, targetPosition, animate);
    }

    IEnumerator MoveCardInternal(AbstractCard card, CardPosition position, bool animate)
    {
      switch (position.PositionCase)
      {
        case CardPosition.PositionOneofCase.Room:
          card.SetRenderingMode(AbstractCard.RenderingMode.Arena);
          return _registry.ArenaService.AddToRoom(card, position.Room, animate);
        case CardPosition.PositionOneofCase.Item:
          card.SetRenderingMode(AbstractCard.RenderingMode.Arena);
          return _registry.ArenaService.AddAsItem(card, position.Item, animate);
        case CardPosition.PositionOneofCase.Staging:
          card.SetRenderingMode(AbstractCard.RenderingMode.Default);
          return _registry.CardStaging.AddCard(card, animate);
        case CardPosition.PositionOneofCase.Hand:
          card.SetRenderingMode(AbstractCard.RenderingMode.Default);
          return _registry.HandForPlayer(position.Hand.Owner).AddCard(card, animate);
        case CardPosition.PositionOneofCase.Deck:
          card.SetRenderingMode(AbstractCard.RenderingMode.Default);
          return _registry.DeckForPlayer(position.Deck.Owner).AddCard(card, animate);
        case CardPosition.PositionOneofCase.Discard:
          throw new NotImplementedException();
        case CardPosition.PositionOneofCase.Scored:
          throw new NotImplementedException();
        case CardPosition.PositionOneofCase.Raid:
          card.SetRenderingMode(AbstractCard.RenderingMode.Default);
          return _registry.RaidService.AddToRaid(card, position.Raid, animate);
        case CardPosition.PositionOneofCase.Browser:
          throw new NotImplementedException();
        default:
          throw new ArgumentOutOfRangeException();
      }
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
      card.transform.rotation = _registry.DeckForPlayer(PlayerName.User).transform.rotation;
      card.SortingOrder = SortingOrder.Create(SortingOrder.Type.Staging);
      var initialMoveTarget = new Vector3(
        target.x - 4,
        target.y + 2,
        target.z - 8);

      DOTween.Sequence()
        .Insert(0,
          card.transform.DOMove(initialMoveTarget, 0.5f).SetEase(Ease.OutCubic))
        .Insert(0, card.transform.DOLocalRotate(new Vector3(270, 0, 0), 0.5f))
        .Insert(0.5f, card.transform.DOMove(_registry.CardStagingArea.position, 0.5f).SetEase(Ease.OutCubic))
        .Insert(0.5f, card.transform.DORotateQuaternion(_registry.CardStagingArea.rotation, 1.0f).SetEase(Ease.Linear))
        .AppendCallback(() => card.StagingAnimationComplete = true);
    }

    Vector3 DeckSpawnPosition(PlayerName playerName) =>
      _registry.DeckForPlayer(playerName).transform.position + new Vector3(0f, 1f, 0f);
  }
}