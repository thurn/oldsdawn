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
  public sealed class ObjectPositionService : MonoBehaviour
  {
    const float CardScale = 1.5f;

    [SerializeField] Registry _registry = null!;
    [SerializeField] Card _cardPrefab = null!;

    readonly Dictionary<GameObjectId, Displayable> _cards = new();

    static readonly GameObjectId UserCardId = new()
    {
      CardId = new CardId
      {
        IdentityCard = PlayerName.User
      }
    };

    static readonly GameObjectId OpponentCardId = new()
    {
      CardId = new CardId
      {
        IdentityCard = PlayerName.Opponent
      }
    };

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

      _cards[UserCardId] = _registry.IdentityCardForPlayer(PlayerName.User);
      _cards[OpponentCardId] = _registry.IdentityCardForPlayer(PlayerName.Opponent);
    }

    public Displayable Find(GameObjectId id) => CheckExists(id);

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
        StartCoroutine(MoveObjectInternal(card, command.Position, animate: false));

        switch (command.Animation)
        {
          case CardCreationAnimation.UserDeckToStaging:
            AnimateFromDeckToStaging(card);
            waitForStaging = true;
            break;
        }
      }

      card.Render(_registry, command.Card, GameContext.Staging, animate: !command.DisableAnimation);
      _cards[ToGameObjectId(command.Card.CardId)] = card;

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
      _cards[ToGameObjectId(cardView.CardId)] = card;
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

    public IEnumerator HandleMoveGameObjectCommand(MoveGameObjectCommand command)
    {
      var card = Find(command.Id);
      if (card.Parent)
      {
        card.Parent!.RemoveObjectIfPresent(card, !command.DisableAnimation);
      }

      return MoveGameObject(Find(command.Id), command.Position, !command.DisableAnimation);
    }

    public IEnumerator MoveGameObject(
      Displayable displayable,
      ObjectPosition targetPosition,
      bool animate = true,
      bool animateRemove = true)
    {
      if (displayable.Parent)
      {
        displayable.Parent!.RemoveObjectIfPresent(displayable, animateRemove);
      }

      return MoveObjectInternal(displayable, targetPosition, animate);
    }

    static GameObjectId ToGameObjectId(CardId cardId) => new()
    {
      CardId = cardId
    };

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

    IEnumerator MoveObjectInternal(Displayable displayable, ObjectPosition position, bool animate)
    {
      switch (position.PositionCase)
      {
        case ObjectPosition.PositionOneofCase.Offscreen:
          displayable.transform.position = Vector3.zero;
          return CollectionUtils.Yield();
        case ObjectPosition.PositionOneofCase.Room:
          return _registry.ArenaService.AddToRoom(displayable, position.Room, animate);
        case ObjectPosition.PositionOneofCase.Item:
          return _registry.ArenaService.AddAsItem(displayable, position.Item, animate);
        case ObjectPosition.PositionOneofCase.Staging:
          return _registry.CardStaging.AddObject(displayable, animate);
        case ObjectPosition.PositionOneofCase.Hand:
          return _registry.HandForPlayer(position.Hand.Owner).AddObject(displayable, animate);
        case ObjectPosition.PositionOneofCase.Deck:
          return _registry.DeckForPlayer(position.Deck.Owner).AddObject(displayable, animate);
        case ObjectPosition.PositionOneofCase.DeckContainer:
          return _registry.DeckPositionForPlayer(position.DeckContainer.Owner).AddObject(displayable, animate);
        case ObjectPosition.PositionOneofCase.DiscardPile:
          return _registry.DiscardPileForPlayer(position.DiscardPile.Owner).AddObject(displayable, animate);
        case ObjectPosition.PositionOneofCase.DiscardPileContainer:
          return _registry.DiscardPilePositionForPlayer(position.DiscardPileContainer.Owner)
            .AddObject(displayable, animate);
        case ObjectPosition.PositionOneofCase.Scored:
          return _registry.CardScoring.AddObject(displayable, animate);
        case ObjectPosition.PositionOneofCase.Raid:
          return _registry.RaidService.AddToRaid(displayable, position.Raid, animate);
        case ObjectPosition.PositionOneofCase.Browser:
          return _registry.CardBrowser.AddObject(displayable, animate);
        case ObjectPosition.PositionOneofCase.Identity:
          return _registry.IdentityCardForPlayer(position.Identity.Owner).AddObject(displayable, animate);
        case ObjectPosition.PositionOneofCase.IdentityContainer:
          return _registry.IdentityCardPositionForPlayer(position.IdentityContainer.Owner)
            .AddObject(displayable, animate);
        default:
          throw new ArgumentOutOfRangeException();
      }
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