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
using System.Linq;
using DG.Tweening;
using Spelldawn.Game;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class ObjectPositionService : MonoBehaviour
  {
    const float CardScale = 1.5f;

    [SerializeField] Registry _registry = null!;
    [SerializeField] Card _cardPrefab = null!;

    readonly Dictionary<GameObjectId, Displayable> _cards = new();

    Card? _optimisticCard;
    SpriteAddress? _userCardBack;

    public void Initialize(CardView? userCard, CardView? opponentCard)
    {
      if (userCard != null)
      {
        SetCardBacks(userCard, PlayerName.User);
        _userCardBack = userCard.CardBack;
      }

      if (opponentCard != null)
      {
        SetCardBacks(opponentCard, PlayerName.Opponent);

        foreach (var spriteRenderer in _registry.DeckForPlayer(PlayerName.Opponent)
                   .GetComponentsInChildren<SpriteRenderer>())
        {
          spriteRenderer.sprite = _registry.AssetService.GetSprite(opponentCard.CardBack);
        }
      }

      _cards[IdUtil.UserCardId] = _registry.IdentityCardForPlayer(PlayerName.User);
      _cards[IdUtil.OpponentCardId] = _registry.IdentityCardForPlayer(PlayerName.Opponent);
    }

    public Displayable Find(GameObjectId id) => CheckExists(id);

    public Card FindCard(CardId id) => (Card)Find(IdUtil.CardObjectId(id));

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
      }, GameContext.Staging);
      _optimisticCard.transform.localScale = new Vector3(CardScale, CardScale, 1f);
      AnimateFromDeckToStaging(_optimisticCard);
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

        StartCoroutine(ObjectDisplayForPosition(command.Position).AddObject(card, animate: false));
        //StartCoroutine(MoveObjectInternal(card, command.Position, animate: false));

        switch (command.Animation)
        {
          case CardCreationAnimation.UserDeckToStaging:
            AnimateFromDeckToStaging(card);
            waitForStaging = true;
            break;
        }
      }

      card.Render(_registry, command.Card, GameContext.Staging, animate: !command.DisableAnimation);
      _cards[IdUtil.CardObjectId(command.Card)] = card;

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
      _cards[IdUtil.CardObjectId(cardView)] = card;
      return card;
    }

    public IEnumerator HandleFireProjectileCommand(FireProjectileCommand command)
    {
      var source = CheckExists(command.SourceId);
      var target = CheckExists(command.TargetId);
      var originalPosition = source.transform.position;
      var originalRotation = source.transform.rotation.eulerAngles;

      if (source.GameContext.IsArenaContext())
      {
        // Enlarge before firing
        yield return TweenUtils.Sequence("EnlargeBeforeFiring")
          .Insert(0, source.transform.DORotate(new Vector3(280, 0, 0), 0.2f))
          .Insert(0,
            source.transform.DOMove(
              Vector3.MoveTowards(source.transform.position, _registry.MainCamera.transform.position, 20f), 0.2f))
          .WaitForCompletion();
      }

      var projectile = _registry.AssetPoolService.Create(
        _registry.AssetService.GetProjectile(command.Projectile), source.transform.position);

      var startPosition = source.transform.position;
      var throwSequence = TweenUtils.Sequence("ProjectileThrow")
        .Insert(0, source.transform.DOMove(Vector3.Lerp(startPosition, target.transform.position, 0.1f), 0.1f))
        .Insert(0.1f, source.transform.DOMove(startPosition, 0.1f));

      if (source.GameContext.IsArenaContext())
      {
        throwSequence
          .Insert(0.8f, source.transform.DOMove(originalPosition, 0.1f))
          .Insert(0.8f, source.transform.DORotate(originalRotation, 0.1f));
      }

      yield return projectile.Fire(
        _registry,
        target.transform,
        command.TravelDuration,
        command.AdditionalHit,
        command.AdditionalHitDelay);

      if (command.HideOnHit)
      {
        target.gameObject.transform.position = Vector3.zero;
      }

      if (command.WaitDuration != null)
      {
        yield return new WaitForSeconds(DataUtils.ToSeconds(command.WaitDuration, 0));
      }

      if (command.JumpToPosition != null)
      {
        yield return MoveGameObject(target, command.JumpToPosition, animate: false, animateRemove: true);
      }

      if (throwSequence.IsActive())
      {
        yield return throwSequence.WaitForCompletion();
      }
    }

    public IEnumerator HandleUpdateCardCommand(UpdateCardCommand command)
    {
      var card = (Card)CheckExists(new GameObjectId { CardId = command.Card.CardId });
      yield return card.Render(_registry, command.Card).WaitForCompletion();
    }

    public IEnumerator HandleMoveGameObjectsCommand(MoveGameObjectsCommand command)
    {
      return MoveGameObjects(command.Ids.Select(Find), command.Position, command.Index, !command.DisableAnimation);
    }

    public IEnumerator MoveGameObject(
      Displayable displayable,
      ObjectPosition targetPosition,
      int? index = null,
      bool animate = true,
      bool animateRemove = true) =>
      MoveGameObjects(CollectionUtils.Once(displayable), targetPosition, index, animate, animateRemove);

    public IEnumerator MoveGameObjects(
      IEnumerable<Displayable> displayable,
      ObjectPosition targetPosition,
      int? index = null,
      bool animate = true,
      bool animateRemove = true)
    {
      return ObjectDisplayForPosition(targetPosition).AddObjects(displayable, animate, index, animateRemove);
    }

    Displayable CheckExists(GameObjectId gameObjectId)
    {
      switch (gameObjectId.IdCase)
      {
        case GameObjectId.IdOneofCase.CardId:
          Errors.CheckState(_cards.ContainsKey(gameObjectId), $"Card not found: {gameObjectId}");
          return _cards[gameObjectId];
        case GameObjectId.IdOneofCase.Deck:
          return _registry.DeckForPlayer(gameObjectId.Deck);
        case GameObjectId.IdOneofCase.Hand:
          return _registry.HandForPlayer(gameObjectId.Hand);
        case GameObjectId.IdOneofCase.DiscardPile:
          return _registry.DiscardPileForPlayer(gameObjectId.DiscardPile);
        default:
          throw new ArgumentOutOfRangeException();
      }
    }

    ObjectDisplay ObjectDisplayForPosition(ObjectPosition position)
    {
      return position.PositionCase switch
      {
        ObjectPosition.PositionOneofCase.Offscreen =>
          _registry.OffscreenCards,
        ObjectPosition.PositionOneofCase.Room =>
          _registry.ArenaService.FindRoom(position.Room.RoomId).ObjectDisplayForLocation(position.Room.RoomLocation),
        ObjectPosition.PositionOneofCase.Item =>
          _registry.ArenaService.ObjectDisplayForLocation(position.Item.ItemLocation),
        ObjectPosition.PositionOneofCase.Staging =>
          _registry.CardStaging,
        ObjectPosition.PositionOneofCase.Hand =>
          _registry.HandForPlayer(position.Hand.Owner),
        ObjectPosition.PositionOneofCase.Deck =>
          _registry.DeckForPlayer(position.Deck.Owner),
        ObjectPosition.PositionOneofCase.DeckContainer =>
          _registry.DeckPositionForPlayer(position.DeckContainer.Owner),
        ObjectPosition.PositionOneofCase.DiscardPile =>
          _registry.DiscardPileForPlayer(position.DiscardPile.Owner),
        ObjectPosition.PositionOneofCase.DiscardPileContainer =>
          _registry.DiscardPilePositionForPlayer(position.DiscardPileContainer.Owner),
        ObjectPosition.PositionOneofCase.Scored =>
          _registry.CardScoring,
        ObjectPosition.PositionOneofCase.Raid =>
          _registry.RaidService.RaidParticipants,
        ObjectPosition.PositionOneofCase.Browser =>
          _registry.CardBrowser,
        ObjectPosition.PositionOneofCase.Identity =>
          _registry.IdentityCardForPlayer(position.Identity.Owner),
        ObjectPosition.PositionOneofCase.IdentityContainer =>
          _registry.IdentityCardPositionForPlayer(position.IdentityContainer.Owner),
        _ => throw new ArgumentOutOfRangeException()
      };
    }

    void SetCardBacks(CardView? cardView, PlayerName playerName)
    {
      if (cardView != null)
      {
        _registry.DeckForPlayer(playerName).SetCardBacks(cardView.CardBack);
      }
    }

    void AnimateFromDeckToStaging(Card card)
    {
      var target = DeckSpawnPosition(PlayerName.User);
      card.transform.position = target;
      card.transform.rotation = _registry.DeckForPlayer(PlayerName.User).transform.rotation;
      card.SetGameContext(GameContext.Staging);
      var initialMoveTarget = new Vector3(
        target.x - 4,
        target.y + 2,
        target.z - 8);

      TweenUtils.Sequence("DeckToStaging")
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