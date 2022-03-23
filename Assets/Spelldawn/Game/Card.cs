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
using DG.Tweening;
using Spelldawn.Protos;
using Spelldawn.Services;
using Spelldawn.Utils;
using TMPro;
using TMPro.Examples;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class Card : Displayable
  {
    public const float CardScale = 1.5f;

    static readonly Vector3 ArenaCardOffset = new(0, 0.6f, 0);

    [Header("Card")] [SerializeField] SpriteRenderer _cardBack = null!;
    [SerializeField] GameObject _arenaCardBack = null!;
    [SerializeField] Transform _cardFront = null!;
    [SerializeField] GameObject _cardShadow = null!;
    [SerializeField] Transform _arenaCard = null!;
    [SerializeField] SpriteRenderer _image = null!;
    [SerializeField] SpriteRenderer _frame = null!;
    [SerializeField] SpriteRenderer _titleBackground = null!;
    [SerializeField] MeshRenderer _outline = null!;
    [SerializeField] TextMeshPro _title = null!;
    [SerializeField] TextMeshPro _rulesText = null!;
    [SerializeField] SpriteRenderer _jewel = null!;
    [SerializeField] SpriteRenderer _arenaFrame = null!;
    [SerializeField] GameObject _arenaShadow = null!;
    [SerializeField] WarpTextExample _warpText = null!;
    [SerializeField] Transform _topLeftAnchor = null!;
    [SerializeField] Transform _topRightAnchor = null!;
    [SerializeField] Transform _bottomLeftAnchor = null!;
    [SerializeField] Transform _bottomRightAnchor = null!;
    [SerializeField] Icon _topLeftIcon = null!;
    [SerializeField] Icon _topRightIcon = null!;
    [SerializeField] Icon _bottomRightIcon = null!;
    [SerializeField] Icon _bottomLeftIcon = null!;
    [SerializeField] Icon _arenaIcon = null!;
    [SerializeField] bool _isRevealed;
    [SerializeField] float _dragStartScreenZ;
    [SerializeField] Vector3 _dragStartPosition;
    [SerializeField] Vector3 _dragOffset;
    [SerializeField] Quaternion _initialDragRotation;
    [SerializeField] ObjectDisplay? _previousParent;

    CardIdentifier? _cardId;
    bool? _serverCanPlay;
    bool? _serverRevealedInArena;
    bool? _isRoomTargeted;
    ObjectPosition? _releasePosition;
    Node? _supplementalInfo;
    Registry _registry = null!;

    [Serializable]
    public sealed class Icon
    {
      [SerializeField] SpriteRenderer _background = null!;
      public SpriteRenderer Background => _background;
      [SerializeField] TextMeshPro _text = null!;
      public TextMeshPro Text => _text;
    }

    public bool IsRevealed => _isRevealed;

    public bool StagingAnimationComplete { get; set; }

    bool InHand() => HasGameContext && GameContext == GameContext.Hand;

    public override float DefaultScale => CardScale;

    public ObjectPosition? ReleasePosition => _releasePosition;

    public Transform TopLeftAnchor => _topLeftAnchor;

    public Transform TopRightAnchor => _topRightAnchor;

    public Transform BottomLeftAnchor => _bottomLeftAnchor;

    public Transform BottomRightAnchor => _bottomRightAnchor;

    public Node? SupplementalInfo => _supplementalInfo;

    public Sequence? Render(
      Registry registry,
      CardView cardView,
      GameContext? gameContext = null,
      bool animate = true)
    {
      _registry = registry;
      _cardId = cardView.CardId;

      if (gameContext is { } gc)
      {
        SetGameContext(gc);
      }

      if (cardView.OwningPlayer != PlayerName.Unspecified)
      {
        _cardBack.sprite = _registry.AssetService.GetSprite(_registry.CardService.GetCardBack(cardView.OwningPlayer));
      }

      _outline.sortingOrder = -1;

      _registry.AssetService.AssignSprite(_arenaFrame, cardView.ArenaFrame);

      if (cardView.RevealedToViewer)
      {
        if (_isRevealed)
        {
          RenderCardView(cardView);
          return null;
        }
        else
        {
          return Flip(_cardFront, _cardBack, () => RenderCardView(cardView), animate);
        }
      }
      else
      {
        if (_isRevealed)
        {
          return Flip(_cardBack, _cardFront, () => RenderCardView(cardView), animate);
        }
        else
        {
          RenderCardView(cardView);
          return null;
        }
      }
    }

    void Update()
    {
      _outline.gameObject.SetActive(CanPlay());
    }

    bool CanPlay() => _serverCanPlay == true && InHand() && _registry.ActionService.CanInitiateAction() && _isRevealed;

    public Card Clone()
    {
      var result = ComponentUtils.GetComponent<Card>(Instantiate(gameObject));
      result._cardId = _cardId;
      result._serverCanPlay = _serverCanPlay;
      result._serverRevealedInArena = _serverRevealedInArena;
      result._isRoomTargeted = _isRoomTargeted;
      result._releasePosition = _releasePosition;
      result._supplementalInfo = _supplementalInfo;
      result._registry = _registry;
      return result;
    }

    protected override void OnSetGameContext(GameContext oldContext, GameContext newContext, int? index = null)
    {
      Errors.CheckNotNull(_registry);
      if (newContext.IsArenaContext())
      {
        _arenaCardBack.SetActive(!_isRevealed);
        _cardBack.maskInteraction = SpriteMaskInteraction.VisibleInsideMask;
        _frame.gameObject.SetActive(false);
        _titleBackground.gameObject.SetActive(false);
        _title.gameObject.SetActive(false);
        _rulesText.gameObject.SetActive(false);
        _jewel.gameObject.SetActive(false);
        _arenaFrame.gameObject.SetActive(true);
        _cardShadow.SetActive(false);
        _arenaShadow.SetActive(true);
        // In Arena mode, we want the image content to be centered within the card, so we shift
        // it around.
        _arenaCard.position = transform.position;
      }
      else
      {
        _arenaCardBack.SetActive(false);
        _cardBack.maskInteraction = SpriteMaskInteraction.None;
        _frame.gameObject.SetActive(true);
        _titleBackground.gameObject.SetActive(true);
        _title.gameObject.SetActive(true);
        _rulesText.gameObject.SetActive(true);
        _jewel.gameObject.SetActive(true);
        _arenaFrame.gameObject.SetActive(false);
        _cardShadow.SetActive(true);
        _arenaShadow.SetActive(false);
        _arenaCard.localPosition = ArenaCardOffset;
      }

      UpdateIcons(null, GameContext.IsArenaContext());
      UpdateRevealedToOpponent(GameContext.IsArenaContext());
    }

    public override bool MouseDown()
    {
      var result = false;

      if (_registry.ActionService.CanInfoZoom(GameContext) && _isRevealed)
      {
        _registry.StaticAssets.PlayCardSound();
        _registry.CardService.DisplayInfoZoom(
          WorldMousePosition(_registry, _registry.MainCamera.WorldToScreenPoint(gameObject.transform.position).z),
          this);
        result = true;
      }

      if (InHand() && CanPlay())
      {
        _registry.CardService.CurrentlyDragging = true;
        SetGameContext(GameContext.Dragging);
        _previousParent = Parent;
        _previousParent!.RemoveObject(this);
        _outline.gameObject.SetActive(false);
        _initialDragRotation = transform.rotation;
        _dragStartScreenZ = _registry.MainCamera.WorldToScreenPoint(gameObject.transform.position).z;
        _dragStartPosition = WorldMousePosition(_registry, _dragStartScreenZ);
        _dragOffset = gameObject.transform.position - _dragStartPosition;
        result = true;
      }

      return result;
    }

    public override void MouseDrag()
    {
      if (!_registry.CardService.CurrentlyDragging)
      {
        return;
      }

      var mousePosition = WorldMousePosition(_registry, _dragStartScreenZ);
      var distanceDragged = Vector2.Distance(mousePosition, _dragStartPosition);
      var t = Mathf.Clamp01(distanceDragged / 5);
      transform.position = _dragOffset + mousePosition;
      var rotation = Quaternion.Slerp(_initialDragRotation, Quaternion.Euler(280, 0, 0), t);
      transform.rotation = rotation;

      if (distanceDragged > 0.5f)
      {
        _registry.CardService.ClearInfoZoom();

        if (_isRoomTargeted == true)
        {
          _registry.ArenaService.ShowRoomSelectorForMousePosition();
        }
      }
      else
      {
        _registry.ArenaService.HideRoomSelector();
      }
    }

    public override void MouseUp()
    {
      _registry.CardService.ClearInfoZoom();

      if (!_registry.CardService.CurrentlyDragging)
      {
        _registry.StaticAssets.PlayCardSound();
        return;
      }

      _registry.CardService.CurrentlyDragging = false;

      if (ShouldReturnToHandOnRelease())
      {
        _registry.StaticAssets.PlayCardSound();
        StartCoroutine(_previousParent!.AddObject(this, animate: true));
        _registry.ArenaService.HideRoomSelector();
        return;
      }

      var action = new PlayCardAction
      {
        CardId = Errors.CheckNotNull(_cardId)
      };

      if (_isRoomTargeted == true)
      {
        action.Target = new CardTarget
        {
          RoomId = Errors.CheckNotDefault(Errors.CheckNotNull(_registry.ArenaService.CurrentRoomSelector).RoomId)
        };
      }

      _registry.ArenaService.HideRoomSelector();

      _registry.ActionService.HandleAction(new GameAction
      {
        PlayCard = action
      });
    }

    static Vector3 WorldMousePosition(Registry registry, float dragStartScreenZ) =>
      registry.MainCamera.ScreenToWorldPoint(
        new Vector3(Input.mousePosition.x, Input.mousePosition.y, dragStartScreenZ));

    bool ShouldReturnToHandOnRelease()
    {
      if (!_registry.ActionService.CanExecuteAction(GameAction.ActionOneofCase.PlayCard))
      {
        return true;
      }

      if (_isRoomTargeted == true)
      {
        return !_registry.ArenaService.CurrentRoomSelector;
      }
      else
      {
        var distance = _dragStartPosition.z - WorldMousePosition(_registry, _dragStartScreenZ).z;
        return distance < 3.5f;
      }
    }

    static Sequence? Flip(Component faceUp, Component faceDown, Action onFlipped, bool animate)
    {
      if (animate)
      {
        const float duration = 0.2f;
        return TweenUtils.Sequence($"{faceUp.transform.parent.gameObject.name} Flip")
          .Insert(atPosition: 0, faceDown.transform.DOLocalRotate(new Vector3(x: 0, y: 90, z: 0), duration))
          .InsertCallback(atPosition: duration, () =>
          {
            faceUp.gameObject.SetActive(value: true);
            faceUp.transform.localRotation = Quaternion.Euler(x: 0, y: -90, z: 0);
            faceDown.gameObject.SetActive(value: false);
            onFlipped();
          })
          .Insert(atPosition: duration, faceUp.transform.DOLocalRotate(Vector3.zero, duration));
      }
      else
      {
        faceUp.gameObject.SetActive(value: true);
        faceDown.gameObject.SetActive(value: false);
        onFlipped();
        return null;
      }
    }

    void RenderCardView(CardView card)
    {
      _serverRevealedInArena = card.RevealedToViewer && card.RevealedToOpponent;

      if (card.RevealedToViewer && card.RevealedCard != null)
      {
        RenderRevealedCard(card.RevealedCard);
      }
      else if (!card.RevealedToViewer)
      {
        RenderHiddenCard();
      }

      UpdateIcons(card.CardIcons, GameContext.IsArenaContext());
      UpdateRevealedToOpponent(GameContext.IsArenaContext());
    }

    void RenderRevealedCard(RevealedCardView revealed)
    {
      _isRevealed = true;

      if (revealed.Title?.Text != null)
      {
        gameObject.name = revealed.Title.Text;
      }

      _serverCanPlay = revealed.CanPlay;

      if (revealed.Targeting?.TargetingCase is { } targeting)
      {
        _isRoomTargeted = targeting == CardTargeting.TargetingOneofCase.PickRoom;
      }

      if (revealed.OnReleasePosition is { } position)
      {
        _releasePosition = position;
      }

      if (revealed.SupplementalInfo is { } info)
      {
        _supplementalInfo = info;
      }

      _cardBack.gameObject.SetActive(value: false);
      _cardFront.gameObject.SetActive(value: true);
      _registry.AssetService.AssignSprite(_image, revealed.Image);
      _image.gameObject.SetActive(true);
      _registry.AssetService.AssignSprite(_frame, revealed.CardFrame);
      _registry.AssetService.AssignSprite(_titleBackground, revealed.TitleBackground);
      SetTitle(revealed.Title?.Text);
      if (revealed.RulesText?.Text != null)
      {
        _rulesText.text = revealed.RulesText.Text;
      }

      _registry.AssetService.AssignSprite(_jewel, revealed.Jewel);
    }

    void RenderHiddenCard()
    {
      _isRevealed = false;
      gameObject.name = "Hidden Card";
      _cardBack.gameObject.SetActive(value: true);
      _cardFront.gameObject.SetActive(value: false);
    }

    void UpdateIcons(CardIcons? cardIcons, bool inArena)
    {
      SetCardIcon(_topLeftIcon, cardIcons?.TopLeftIcon, !inArena);
      SetCardIcon(_topRightIcon, cardIcons?.TopRightIcon, !inArena);
      SetCardIcon(_bottomRightIcon, cardIcons?.BottomRightIcon, !inArena);
      SetCardIcon(_bottomLeftIcon, cardIcons?.BottomLeftIcon, !inArena);
      SetCardIcon(_arenaIcon, cardIcons?.ArenaIcon, inArena);
    }

    void UpdateRevealedToOpponent(bool inArena)
    {
      if (inArena && _serverRevealedInArena != true)
      {
        _image.color = Color.gray;
        _arenaFrame.color = Color.gray;
      }
      else
      {
        _image.color = Color.white;
        _arenaFrame.color = Color.white;
      }
    }

    void SetCardIcon(Icon icon, CardIcon? cardIcon, bool show)
    {
      _registry.AssetService.AssignSprite(icon.Background, cardIcon?.Background);

      if (cardIcon?.BackgroundScale is { } scale)
      {
        icon.Background.transform.localScale = scale * Vector3.one;
      }

      if (cardIcon?.Text != null)
      {
        icon.Text.text = cardIcon.Text;
      }

      var iconContainer = icon.Background.transform.parent;
      iconContainer.gameObject.SetActive(show);
    }

    void SetTitle(string? title)
    {
      if (title == null)
      {
        return;
      }

      _title.text = title;
      var length = title.Length;

      if (length < 10)
      {
        _title.transform.localPosition = new Vector3(0, 1.95f, 0);
        _warpText.enabled = false;
      }
      else
      {
        _warpText.enabled = true;

        switch (length)
        {
          case < 14:
            _warpText.CurveScale = 0.75f;
            _title.transform.localPosition = new Vector3(0, 1.93f, 0);
            break;
          case < 17:
            _warpText.CurveScale = 1.5f;
            _title.transform.localPosition = new Vector3(0, 1.89f, 0);
            break;
          case < 20:
            _warpText.CurveScale = 1.7f;
            _title.transform.localPosition = new Vector3(0, 1.87f, 0);
            break;
          default:
            _warpText.CurveScale = 1.9f;
            _title.transform.localPosition = new Vector3(0, 1.87f, 0);
            break;
        }

        var internalText = ComponentUtils.GetComponent<TMP_Text>(_title);
        StartCoroutine(_warpText.WarpText(internalText));
      }
    }
  }
}