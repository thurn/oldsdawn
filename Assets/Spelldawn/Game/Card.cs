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
    [SerializeField] bool _canPlay;
    [SerializeField] float _dragStartScreenZ;
    [SerializeField] Vector3 _dragStartPosition;
    [SerializeField] Vector3 _dragOffset;
    [SerializeField] Quaternion _initialDragRotation;
    [SerializeField] ObjectDisplay? _previousParent;
    [SerializeField] uint _previousParentIndex;

    Registry _registry = null!;

    CardView? _cardView;
    public CardView? CardView => _cardView;

    RevealedCardView? _revealedCardView;
    public RevealedCardView? RevealedCardView => _revealedCardView;

    RoomIdentifier? _targetRoom;
    public RoomIdentifier? TargetRoom => _targetRoom;

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

    public Transform TopLeftAnchor => _topLeftAnchor;

    public Transform TopRightAnchor => _topRightAnchor;

    public Transform BottomLeftAnchor => _bottomLeftAnchor;

    public Transform BottomRightAnchor => _bottomRightAnchor;

    public Sequence? Render(
      Registry registry,
      CardView cardView,
      GameContext? gameContext = null,
      bool animate = true)
    {
      if (gameContext is { } gc)
      {
        SetGameContext(gc);
      }

      _registry = registry;
      if (cardView.OwningPlayer != PlayerName.Unspecified)
      {
        _cardBack.sprite = _registry.AssetService.GetSprite(_registry.CardService.GetCardBack(cardView.OwningPlayer));
      }

      _outline.sortingOrder = -1;
      _cardView = cardView;

      if (cardView.ArenaFrame != null)
      {
        _arenaFrame.sprite = registry.AssetService.GetSprite(cardView.ArenaFrame);
      }

      if (cardView.RevealedCard != null)
      {
        if (_isRevealed)
        {
          RenderRevealedCard(cardView);
          return null;
        }
        else
        {
          return Flip(_cardFront, _cardBack, () => RenderRevealedCard(cardView), animate);
        }
      }
      else
      {
        if (_isRevealed)
        {
          return Flip(_cardBack, _cardFront, RenderHiddenCard, animate);
        }
        else
        {
          RenderHiddenCard();
          return null;
        }
      }
    }

    void Update()
    {
      if (InHand() &&
          _registry.ActionService.CanInitiateAction() &&
          _revealedCardView is { Cost: { } cost })
      {
        var canPlayCard =
          cost.CanPlay &&
          cost.ManaCost <= _registry.ManaDisplayForPlayer(PlayerName.User).CurrentMana &&
          cost.ActionCost <= _registry.ActionDisplayForPlayer(PlayerName.User).AvailableActions;
        _canPlay = cost.CanPlayAlgorithm switch
        {
          CanPlayAlgorithm.Optimistic => canPlayCard,
          CanPlayAlgorithm.AdditionalCost when !canPlayCard => false,
          CanPlayAlgorithm.AdditionalPlay when canPlayCard => true,
          _ => _canPlay
        };
      }
      else
      {
        _canPlay = false;
      }

      _outline.gameObject.SetActive(_canPlay);
    }

    protected override void OnSetGameContext(GameContext oldContext, GameContext newContext, int? index = null)
    {
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

      UpdateIcons(GameContext.IsArenaContext());
      UpdateRevealedToOpponent(GameContext.IsArenaContext());

      if (_revealedCardView != null)
      {
        // TODO: For some reason this is needed to fix the text curve, figure out why
        SetTitle(_revealedCardView.Title.Text);
      }
    }

    public override bool MouseDown()
    {
      var result = false;

      if (_registry.ActionService.CanInfoZoom(GameContext) && _revealedCardView != null)
      {
        _registry.StaticAssets.PlayCardSound();
        _registry.CardService.DisplayInfoZoom(
          WorldMousePosition(_registry, _registry.MainCamera.WorldToScreenPoint(gameObject.transform.position).z),
          this);
        result = true;
      }

      if (InHand() && _canPlay)
      {
        _registry.CardService.CurrentlyDragging = true;
        SetGameContext(GameContext.Dragging);
        _previousParent = Parent;
        _previousParentIndex = _previousParent!.RemoveObject(this);
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
      }

      if (_revealedCardView?.Targeting?.TargetingCase == CardTargeting.TargetingOneofCase.PickRoom)
      {
        _targetRoom = _registry.ArenaService.ShowRoomSelectorForMousePosition();
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
      _registry.ArenaService.HideRoomSelector();

      var distance = _dragStartPosition.z - WorldMousePosition(_registry, _dragStartScreenZ).z;
      if (distance < 3.5f ||
          !_registry.ActionService.CanExecuteAction(GameAction.ActionOneofCase.PlayCard) ||
          (_revealedCardView?.Targeting?.TargetingCase == CardTargeting.TargetingOneofCase.PickRoom &&
           _targetRoom == null))
      {
        // Return to hand
        _registry.StaticAssets.PlayCardSound();
        StartCoroutine(_previousParent!.AddObject(this, animate: true, index: _previousParentIndex));
      }
      else
      {
        var action = new PlayCardAction
        {
          CardId = _cardView!.CardId
        };

        if (_targetRoom is { } room)
        {
          action.Target = new CardTarget
          {
            RoomId = room
          };
        }

        _registry.ActionService.HandleAction(new GameAction
        {
          PlayCard = action
        });
      }
    }

    static Vector3 WorldMousePosition(Registry registry, float dragStartScreenZ) =>
      registry.MainCamera.ScreenToWorldPoint(
        new Vector3(Input.mousePosition.x, Input.mousePosition.y, dragStartScreenZ));

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
      UpdateIcons(GameContext.IsArenaContext());
      UpdateRevealedToOpponent(GameContext.IsArenaContext());
    }

    void UpdateIcons(bool inArena)
    {
      SetCardIcon(_topLeftIcon, _cardView?.CardIcons?.TopLeftIcon, !inArena);
      SetCardIcon(_topRightIcon, _cardView?.CardIcons?.TopRightIcon, !inArena);
      SetCardIcon(_bottomRightIcon, _cardView?.CardIcons?.BottomRightIcon, !inArena);
      SetCardIcon(_bottomLeftIcon, _cardView?.CardIcons?.BottomLeftIcon, !inArena);
      SetCardIcon(_arenaIcon, _cardView?.CardIcons?.ArenaIcon, inArena);
    }

    void UpdateRevealedToOpponent(bool inArena)
    {
      if (inArena && _revealedCardView?.RevealedInArena == false)
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
      if (cardIcon != null && show)
      {
        icon.Background.transform.parent.gameObject.SetActive(true);
        icon.Background.sprite = _registry.AssetService.GetSprite(cardIcon.Background);
        icon.Background.transform.localScale =
          (cardIcon.BackgroundScale == 0f ? 1f : cardIcon.BackgroundScale) * Vector3.one;
        icon.Text.text = cardIcon.Text;
      }
      else
      {
        icon.Background.transform.parent.gameObject.SetActive(false);
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