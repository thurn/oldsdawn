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
  public sealed class CardService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] BoxCollider _playCardArea = null!;
    [SerializeField] Card _cardPrefab = null!;
    [SerializeField] Card _tokenCardPrefab = null!;
    [SerializeField] ObjectDisplay _infoZoomLeft = null!;
    [SerializeField] ObjectDisplay _infoZoomRight = null!;

    Card? _optimisticCard;
    SpriteAddress _userCardBack = null!;
    SpriteAddress _opponentCardBack = null!;

    readonly List<Sequence> _optimisticAnimations = new();
    readonly RaycastHit[] _raycastHitsTempBuffer = new RaycastHit[8];
    readonly Dictionary<CardIdentifier, Card> _cards = new();

    public bool CurrentlyDragging { get; set; }

    public IEnumerator Sync(List<CardView> views, GameObjectPositions? positions, bool animate = true,
      bool delete = true)
    {
      var toDelete = _cards.Keys.ToHashSet();
      var coroutines = new List<Coroutine>();

      foreach (var view in views)
      {
        toDelete.Remove(view.CardId);
        Card card;
        var optimistic = false;

        if (_cards.ContainsKey(view.CardId))
        {
          card = _cards[view.CardId];
        }
        else if (_optimisticCard)
        {
          // When a user takes the 'draw card' action, we start the animation before the server responds in
          // order to make it feel smooth.
          card = _optimisticCard!;
          _optimisticCard = null;
          _cards[view.CardId] = card;
          optimistic = true;
        }
        else
        {
          card = InstantiateCardPrefab(view.Prefab);
          _cards[view.CardId] = card;
          card.transform.localScale = new Vector3(Card.CardScale, Card.CardScale, 1f);
          var position = view.CreatePosition ?? Errors.CheckNotNull(view.CardPosition);
          _registry.ObjectPositionService.MoveGameObjectImmediate(card, position);
        }

        card.Render(view, animate: animate);

        if (!optimistic)
        {
          // Need to update all cards in case sorting keys change
          coroutines.Add(StartCoroutine(
            _registry.ObjectPositionService.MoveGameObject(card, view.CardPosition, animate)));
        }
      }

      if (positions != null)
      {
        coroutines.Add(StartCoroutine(_registry.ObjectPositionService.MoveByIdentifier(
          IdUtil.DeckObjectId(PlayerName.User), positions.UserDeck, animate)));
        coroutines.Add(StartCoroutine(_registry.ObjectPositionService.MoveByIdentifier(
          IdUtil.DeckObjectId(PlayerName.Opponent), positions.OpponentDeck, animate)));
        coroutines.Add(StartCoroutine(_registry.ObjectPositionService.MoveByIdentifier(
          IdUtil.IdentityCardId(PlayerName.User), positions.UserIdentity, animate)));
        coroutines.Add(StartCoroutine(_registry.ObjectPositionService.MoveByIdentifier(
          IdUtil.IdentityCardId(PlayerName.Opponent), positions.OpponentIdentity, animate)));
        coroutines.Add(StartCoroutine(_registry.ObjectPositionService.MoveByIdentifier(
          IdUtil.DiscardPileObjectId(PlayerName.User), positions.UserDiscard, animate)));
        coroutines.Add(StartCoroutine(_registry.ObjectPositionService.MoveByIdentifier(
          IdUtil.DiscardPileObjectId(PlayerName.Opponent), positions.OpponentDiscard, animate)));
      }

      if (delete)
      {
        coroutines.AddRange(
          toDelete.Select(cardId => StartCoroutine(HandleDestroyCard(cardId, animate))));
      }

      foreach (var coroutine in coroutines)
      {
        yield return coroutine;
      }

      // Wait for optimistic animations to play out before continuing, to avoid jumping the card around.
      foreach (var sequence in _optimisticAnimations.Where(s => s.IsActive()))
      {
        yield return sequence.WaitForCompletion();
      }
      yield return _registry.RevealedCardsBrowserSmall.WaitUntilIdle();
    }

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
      _optimisticCard.Render(new CardView { OwningPlayer = PlayerName.User }, GameContext.Staging);
      _optimisticCard.transform.localScale = new Vector3(Card.CardScale, Card.CardScale, 1f);
      PlayDrawCardAnimation(_optimisticCard);
    }

    public Card CreateAndAddCard(CardView cardView, GameContext gameContext, bool animate)
    {
      var card = InstantiateCardPrefab(cardView.Prefab);
      card.transform.localScale = new Vector3(Card.CardScale, Card.CardScale, 1f);
      card.Render(cardView, gameContext, animate: animate);
      _cards[Errors.CheckNotNull(cardView.CardId)] = card;
      return card;
    }

    public bool IsMouseOverPlayCardArea()
    {
      var ray = _registry.MainCamera.ScreenPointToRay(Input.mousePosition);
      var hits = Physics.RaycastNonAlloc(ray, _raycastHitsTempBuffer, 100);
      for (var i = 0; i < hits; ++i)
      {
        var hit = _raycastHitsTempBuffer[i];
        if (hit.collider == _playCardArea)
        {
          return true;
        }
      }

      return false;
    }

    public void DisplayInfoZoom(Vector3 worldMousePosition, Card card)
    {
      StartCoroutine(InfoZoom(worldMousePosition, card));
    }

    IEnumerator InfoZoom(Vector3 worldMousePosition, Card card)
    {
      ClearInfoZoom();

      var zoomed = InfoCopy(card);
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

    /// <summary>
    /// Creates a clone of a card for display at large size
    /// </summary>
    public static Card InfoCopy(Card card)
    {
      var zoomed = card.Clone();
      zoomed.transform.localScale = new Vector3(Card.CardScale, Card.CardScale, 1f);
      zoomed.SetGameContext(GameContext.InfoZoom);
      zoomed.gameObject.name = $"{card.name} Info";
      return zoomed;
    }

    public void ClearInfoZoom()
    {
      _registry.DocumentService.ClearSupplementalCardInfo();
      _infoZoomLeft.DestroyAll();
      _infoZoomRight.DestroyAll();
    }

    public IEnumerator HandleDestroyCard(CardIdentifier cardId, bool animate)
    {
      var card = FindCard(cardId);
      var sequence = card.TurnFaceDown(animate);

      if (card.DestroyPosition != null)
      {
        yield return _registry.ObjectPositionService.MoveGameObject(card, card.DestroyPosition, animate);
      }

      _cards.Remove(cardId);

      if (card.Parent)
      {
        var parent = card.Parent;
        parent!.RemoveObjectIfPresent(card, animate: false);
      }

      if (sequence != null && sequence.IsActive() && !sequence.IsComplete())
      {
        yield return sequence.WaitForCompletion();
      }

      Destroy(card.gameObject);
    }


    /// <summary>Plays a custom animation specifically for the 'Draw Card' action.</summary>
    void PlayDrawCardAnimation(Card card)
    {
      var target = DeckSpawnPosition(PlayerName.User);
      card.transform.position = target;
      card.transform.rotation = _registry.DeckForPlayer(PlayerName.User).transform.rotation;
      card.SetGameContext(GameContext.Staging);
      var initialMoveTarget = new Vector3(
        target.x - 4,
        target.y + 2,
        target.z - 8);
      
      var sequence = TweenUtils.Sequence("DrawCardAnimation")
        .Insert(0,
          card.transform.DOMove(initialMoveTarget, 0.5f).SetEase(Ease.OutCubic))
        .Insert(0, card.transform.DOLocalRotate(new Vector3(270, 0, 0), 0.5f))
        .InsertCallback(0.5f, () =>
          {
            _registry.StaticAssets.PlayDrawCardSound();
            StartCoroutine(_registry.RevealedCardsBrowserSmall.AddObject(card));
          }
        );
      _optimisticAnimations.Add(sequence);
    }

    Vector3 DeckSpawnPosition(PlayerName playerName) =>
      _registry.DeckForPlayer(playerName).transform.position + new Vector3(0f, 1f, 0f);

    Card InstantiateCardPrefab(CardPrefab prefab)
    {
      var result = ComponentUtils.Instantiate(prefab switch
      {
        CardPrefab.Standard => _cardPrefab,
        CardPrefab.TokenCard => _tokenCardPrefab,
        _ => throw new ArgumentOutOfRangeException()
      });
      result.Registry = _registry;
      return result;
    }
  }
}