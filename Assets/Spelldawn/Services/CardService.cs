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
    [SerializeField] Deck _userDeck = null!;
    [SerializeField] Deck _opponentDeck = null!;
    [SerializeField] Transform _userDiscardPile = null!;
    [SerializeField] Transform _opponentDiscardPile = null!;
    [SerializeField] Card _cardPrefab = null!;
    [SerializeField] Transform _cardStagingArea = null!;

    readonly Dictionary<CardId, Card> _cards = new();
    Card? _optimisticCard;
    SpriteAddress? _userCardBack;

    Deck PlayerDeck(PlayerName playerName) =>
      playerName == PlayerName.User ? _userDeck : _opponentDeck;

    Transform PlayerDiscardPile(PlayerName playerName) =>
      playerName == PlayerName.User ? _userDiscardPile : _opponentDiscardPile;

    public void Initialize(CardView? userCard)
    {
      _userCardBack = userCard?.CardBack;
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

      Card card;
      if (_optimisticCard)
      {
        card = _optimisticCard!;
        _optimisticCard = null;
      }
      else
      {
        card = ComponentUtils.Instantiate(_cardPrefab);
        card.transform.localScale = new Vector3(CardScale, CardScale, 1f);
        switch (command.Position)
        {
          case CreateCardPosition.UserDeck:
            AnimateFromDeckToStaging(card);
            break;
          case CreateCardPosition.Spawn:
            card.transform.position = _cardStagingArea.position;
            break;
        }
      }

      card.Render(_registry, command.Card);
      _cards[command.Card.CardId] = card;

      yield return new WaitUntil(() => card.IsRevealed && card.StagingAnimationComplete);
      yield return new WaitForSeconds(0.5f);
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
          return MoveTo(card, PlayerDeck(command.TargetPlayer).transform);
        case GameZone.Discard:
          return MoveTo(card, PlayerDiscardPile(command.TargetPlayer));
        case GameZone.Banished:
          Destroy(card);
          break;
      }

      return CollectionUtils.Yield();
    }

    IEnumerator<YieldInstruction> AddToHand(Hand hand, Card card)
    {
      Debug.Log($"AddToHand:");
      yield break;
    }

    IEnumerator<YieldInstruction> MoveTo(Card card, Transform target)
    {
      yield return DOTween.Sequence()
        .Insert(0, card.transform.DOMove(target.position, 0.5f))
        .Insert(0, card.transform.DORotateQuaternion(target.rotation, 0.5f))
        .WaitForCompletion();
    }

    void AnimateFromDeckToStaging(Card card)
    {
      card.transform.position = _userDeck.transform.position;
      var initialMoveTarget = new Vector3(
        _userDeck.transform.position.x - 4,
        _userDeck.transform.position.y + 2,
        _userDeck.transform.position.z - 8);

      DOTween.Sequence()
        .Insert(0,
          card.transform.DOMove(initialMoveTarget, 0.5f).SetEase(Ease.OutCubic))
        .Insert(0.5f, card.transform.DOMove(_cardStagingArea.position, 0.5f).SetEase(Ease.OutCubic))
        .Insert(0, card.transform.DORotateQuaternion(_cardStagingArea.rotation, 1.0f).SetEase(Ease.Linear))
        .AppendCallback(() => card.StagingAnimationComplete = true);
    }
  }
}