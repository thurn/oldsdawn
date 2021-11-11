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
using UnityEngine.Serialization;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class Card : Displayable
  {
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
    [SerializeField] Transform _uiAnchor = null!;
    [SerializeField] Icon _topLeftIcon = null!;
    [SerializeField] Icon _topRightIcon = null!;
    [SerializeField] Icon _bottomRightIcon = null!;
    [SerializeField] Icon _bottomLeftIcon = null!;
    [FormerlySerializedAs("_centerIcon")] [SerializeField] Icon _arenaIcon = null!;
    [SerializeField] bool _isRevealed;
    [SerializeField] bool _canPlay;
    [SerializeField] bool _isDragging;
    [SerializeField] float _dragStartScreenZ;
    [SerializeField] Vector3 _dragStartPosition;
    [SerializeField] Vector3 _dragOffset;
    [SerializeField] Quaternion _initialDragRotation;
    [SerializeField] int _handIndex;

    Registry _registry = null!;
    CardView? _cardView;
    RevealedCardView? _revealedCardView;
    RoomId? _targetRoom;

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

    public void Render(Registry registry, CardView cardView, GameContext? gameContext = null, bool animate = true)
    {
      if (gameContext is {} gc)
      {
        SetGameContext(gc);
      }

      _registry = registry;
      _cardBack.sprite = _registry.AssetService.GetSprite(cardView.CardBack);
      _outline.sortingOrder = -1;
      _cardView = cardView;

      if (cardView.RevealedCard != null)
      {
        if (_isRevealed)
        {
          RenderRevealedCard(cardView);
        }
        else
        {
          Flip(_cardFront, _cardBack, () => RenderRevealedCard(cardView), animate);
        }
      }
      else
      {
        if (_isRevealed)
        {
          Flip(_cardBack, _cardFront, RenderHiddenCard, animate);
        }
        else
        {
          RenderHiddenCard();
        }
      }
    }

    void Update()
    {
      if (!_isDragging && InHand() && _revealedCardView is { Cost: { } cost })
      {
        var haveAvailableResources =
          cost.ManaCost <= _registry.ManaDisplayForPlayer(PlayerName.User).CurrentMana &&
          cost.ActionCost <= _registry.ActionDisplayForPlayer(PlayerName.User).AvailableActions;
        _canPlay = cost.CanPlayAlgorithm switch
        {
          CanPlayAlgorithm.Optimistic => haveAvailableResources,
          CanPlayAlgorithm.AdditionalCost when !haveAvailableResources => false,
          CanPlayAlgorithm.AdditionalPlay when haveAvailableResources => true,
          _ => _canPlay
        };

        _outline.gameObject.SetActive(_canPlay);
      }
      else
      {
        _outline.gameObject.SetActive(false);
      }
    }

    public override Transform InterfaceAnchor() => _uiAnchor;

    protected override void OnSetGameContext(GameContext oldContext, GameContext newContext, int? index = null)
    {
      if (newContext.IsArenaContext())
      {
        _arenaCardBack.SetActive(true);
        _cardBack.maskInteraction = SpriteMaskInteraction.VisibleInsideMask;
        _frame.gameObject.SetActive(false);
        _titleBackground.gameObject.SetActive(false);
        _title.gameObject.SetActive(false);
        _rulesText.gameObject.SetActive(false);
        _jewel.gameObject.SetActive(false);
        _arenaFrame.gameObject.SetActive(true);
        _cardShadow.SetActive(false);
        _arenaShadow.SetActive(true);
        _topLeftIcon.Background.gameObject.SetActive(false);
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

      UpdateIcons();
      if (_revealedCardView != null)
      {
        // TODO: For some reason this is needed to fix the text curve, figure out why
        SetTitle(_revealedCardView.Title.Text);
      }
    }

    void OnMouseDown()
    {
      if (_isDragging)
      {
        // Unity seems to send this event multiple times a lot...
        return;
      }

      if (InHand() && _canPlay)
      {
        _isDragging = true;
        SetGameContext(GameContext.Dragging);
        _handIndex = Parent!.RemoveObject(this);
        _outline.gameObject.SetActive(false);
        _initialDragRotation = transform.rotation;
        _dragStartScreenZ = _registry.MainCamera.WorldToScreenPoint(gameObject.transform.position).z;
        _dragStartPosition = DragWorldMousePosition();
        _dragOffset = gameObject.transform.position - _dragStartPosition;
      }
    }

    void OnMouseDrag()
    {
      if (_isDragging)
      {
        var mousePosition = DragWorldMousePosition();
        var distanceDragged = Vector2.Distance(mousePosition, _dragStartPosition);
        var t = Mathf.Clamp01(distanceDragged / 5);
        transform.position = _dragOffset + mousePosition;
        var rotation = Quaternion.Slerp(_initialDragRotation, Quaternion.Euler(280, 0, 0), t);
        transform.rotation = rotation;

        if (_revealedCardView?.Targeting?.TargetingCase == CardTargeting.TargetingOneofCase.PickRoom)
        {
          _targetRoom = _registry.ArenaService.ShowRoomSelectorForMousePosition();
        }
      }
    }

    void OnMouseUp()
    {
      _registry.ArenaService.HideRoomSelector();
      var distance = _dragStartPosition.z - DragWorldMousePosition().z;
      if (!_isDragging || distance < 3.5f || _revealedCardView?.OnReleasePosition == null)
      {
        // Return to hand
        StartCoroutine(Parent!.AddObject(this, animate: true, index: _handIndex));
      }
      else
      {
        var position = _revealedCardView?.OnReleasePosition;
        if (position?.PositionCase == ObjectPosition.PositionOneofCase.Room)
        {
          if (_targetRoom is { } targetRoom)
          {
            // Move to targeted room if one is available
            var newPosition = new ObjectPosition();
            newPosition.MergeFrom(position);
            newPosition.Room.RoomId = targetRoom;
            position = newPosition;
            _targetRoom = null;
          }
          else
          {
            position = null;
          }
        }

        StartCoroutine(position != null
          ? _registry.ObjectPositionService.MoveGameObject(this, position)
          : Parent!.AddObject(this, animate: true, index: _handIndex));
      }

      _isDragging = false;
    }

    Vector3 DragWorldMousePosition() => _registry.MainCamera.ScreenToWorldPoint(
      new Vector3(Input.mousePosition.x, Input.mousePosition.y, _dragStartScreenZ));

    static void Flip(Component faceUp, Component faceDown, Action onFlipped, bool animate)
    {
      if (animate)
      {
        const float duration = 0.2f;
        TweenUtils.Sequence($"{faceUp.transform.parent.gameObject.name} Flip")
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
      }
    }

    void RenderRevealedCard(CardView card)
    {
      var revealed = card.RevealedCard;
      _revealedCardView = revealed;
      _isRevealed = true;
      _canPlay = card.RevealedCard?.Cost?.CanPlay == true;

      gameObject.name = revealed.Title.Text;
      _cardBack.gameObject.SetActive(value: false);
      _cardFront.gameObject.SetActive(value: true);
      _image.sprite = _registry.AssetService.GetSprite(revealed.Image);
      _image.gameObject.SetActive(true);
      _frame.sprite = _registry.AssetService.GetSprite(revealed.CardFrame);
      _titleBackground.sprite = _registry.AssetService.GetSprite(revealed.TitleBackground);
      SetTitle(revealed.Title.Text);
      _rulesText.text = revealed.RulesText.Text;
      _jewel.sprite = _registry.AssetService.GetSprite(revealed.Jewel);
      UpdateIcons();
    }

    void UpdateIcons()
    {
      SetCardIcon(_topLeftIcon, _cardView?.CardIcons?.TopLeftIcon, !GameContext.IsArenaContext());
      SetCardIcon(_topRightIcon, _cardView?.CardIcons?.TopRightIcon, !GameContext.IsArenaContext());
      SetCardIcon(_bottomRightIcon, _cardView?.CardIcons?.BottomRightIcon, !GameContext.IsArenaContext());
      SetCardIcon(_bottomLeftIcon, _cardView?.CardIcons?.BottomLeftIcon, !GameContext.IsArenaContext());
      SetCardIcon(_arenaIcon, _cardView?.CardIcons?.ArenaIcon, GameContext.IsArenaContext());
    }

    void SetCardIcon(Icon icon, CardIcon? cardIcon, bool show)
    {
      if (cardIcon != null && show)
      {
        icon.Background.gameObject.SetActive(true);
        icon.Background.sprite = _registry.AssetService.GetSprite(cardIcon.Background);
        icon.Text.text = cardIcon.Text;
      }
      else
      {
        icon.Background.gameObject.SetActive(false);
      }
    }

    void SetTitle(string title)
    {
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

    void RenderHiddenCard()
    {
      _isRevealed = false;
      gameObject.name = "Hidden Card";
      _cardBack.gameObject.SetActive(value: true);
      _cardFront.gameObject.SetActive(value: false);
    }
  }
}