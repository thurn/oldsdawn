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
    [SerializeField] Registry _registry = null!;

    public Displayable Find(GameObjectIdentifier id) => CheckExists(id);

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
        command.AdditionalHitDelay,
        command.FireSound,
        command.ImpactSound);

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

    public IEnumerator MoveByIdentifier(GameObjectIdentifier identifier, ObjectPosition position, bool animate) => 
      MoveGameObject(Find(identifier), position, animate);
    
    public IEnumerator HandleMoveGameObjectsCommand(MoveGameObjectsCommand command)
    {
      if (!command.DisableAnimation)
      {
        _registry.StaticAssets.PlayCardSound();
      }

      var coroutines = new List<Coroutine>();

      foreach (var move in command.Moves)
      {
        coroutines.Add(
          StartCoroutine(
            MoveGameObject(Find(move.Id),
              move.Position,
              animate: !command.DisableAnimation
            )));
      }

      foreach (var coroutine in coroutines)
      {
        yield return coroutine;
      }

      if (command.Delay != null)
      {
        yield return new WaitForSeconds(command.Delay.Milliseconds / 1000f);
      }
    }

    public IEnumerator MoveGameObject(
      Displayable displayable,
      ObjectPosition targetPosition,
      bool animate = true,
      bool animateRemove = true)
    {
      displayable.SortingKey = targetPosition.SortingKey;
      displayable.SortingSubkey = targetPosition.SortingSubkey;      
      return ObjectDisplayForPosition(targetPosition).AddObject(displayable, animate, animateRemove);
    }

    public void MoveGameObjectImmediate(Displayable displayable, ObjectPosition targetPosition)
    {
      displayable.SortingKey = targetPosition.SortingKey;
      displayable.SortingSubkey = targetPosition.SortingSubkey;
      ObjectDisplayForPosition(targetPosition).AddObjectImmediate(displayable);
    }

    Displayable CheckExists(GameObjectIdentifier gameObjectId)
    {
      switch (gameObjectId.IdCase)
      {
        case GameObjectIdentifier.IdOneofCase.CardId:
          return _registry.CardService.FindCard(gameObjectId.CardId);
        case GameObjectIdentifier.IdOneofCase.Identity:
          return _registry.IdentityCardForPlayer(gameObjectId.Identity);
        case GameObjectIdentifier.IdOneofCase.Deck:
          return _registry.DeckForPlayer(gameObjectId.Deck);
        case GameObjectIdentifier.IdOneofCase.DiscardPile:
          return _registry.DiscardPileForPlayer(gameObjectId.DiscardPile);
        default:
          throw new ArgumentOutOfRangeException();
      }
    }

    ObjectDisplay ObjectDisplayForPosition(ObjectPosition position)
    {
      Errors.CheckNotNull(position);
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
        ObjectPosition.PositionOneofCase.Raid =>
          _registry.RaidService.RaidParticipants,
        ObjectPosition.PositionOneofCase.Browser =>
          _registry.CardBrowser,
        ObjectPosition.PositionOneofCase.Identity =>
          _registry.IdentityCardForPlayer(position.Identity.Owner),
        ObjectPosition.PositionOneofCase.IdentityContainer =>
          _registry.IdentityCardPositionForPlayer(position.IdentityContainer.Owner),
        ObjectPosition.PositionOneofCase.IntoCard =>
          _registry.CardService.FindCard(position.IntoCard.CardId).ContainedObjects,
        ObjectPosition.PositionOneofCase.Revealed => position.Revealed.Size switch
        {
          RevealedCardsBrowserSize.Small => _registry.RevealedCardsBrowserSmall,
          RevealedCardsBrowserSize.Large => _registry.RevealedCardsBrowserLarge,
          _ => throw new ArgumentOutOfRangeException()
        },
        _ => throw new ArgumentOutOfRangeException()
      };
    }
  }
}