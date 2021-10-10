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
using UnityEngine.Rendering;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class Card : MonoBehaviour
  {
    [SerializeField] SpriteRenderer _cardBack = null!;
    [SerializeField] Transform _cardFront = null!;
    [SerializeField] GameObject _cardShadow = null!;
    [SerializeField] SpriteRenderer _image = null!;
    [SerializeField] SpriteRenderer _frame = null!;
    [SerializeField] SpriteRenderer _titleBackground = null!;
    [SerializeField] MeshRenderer _outline = null!;
    [SerializeField] TextMeshPro _title = null!;
    [SerializeField] TextMeshPro _rulesText = null!;
    [SerializeField] SpriteRenderer _jewel = null!;
    [SerializeField] SpriteRenderer _arenaFrame = null!;
    [SerializeField] GameObject _arenaShadow = null!;
    [SerializeField] SortingGroup _sortingGroup = null!;
    [SerializeField] WarpTextExample _warpText = null!;
    [SerializeField] Icon _topLeftIcon = null!;
    [SerializeField] Icon _topRightIcon = null!;
    [SerializeField] Icon _bottomRightIcon = null!;
    [SerializeField] Icon _bottomLeftIcon = null!;
    [SerializeField] Icon _centerIcon = null!;
    [SerializeField] bool _isRevealed;
    [SerializeField] bool _canPlay;
    [SerializeField] bool _isDragging;
    [SerializeField] bool _interactive;
    [SerializeField] float _dragStartScreenZ;
    [SerializeField] Vector3 _dragStartPosition;
    [SerializeField] Vector3 _dragOffset;
    [SerializeField] Quaternion _initialDragRotation;
    [SerializeField] int _handIndex;
    [SerializeField] RenderingMode _renderingMode;

    Registry _registry = null!;
    CardView? _cardView;
    RevealedCardView? _revealedCardView;

    [Serializable]
    public sealed class Icon
    {
      [SerializeField] SpriteRenderer _background = null!;
      public SpriteRenderer Background => _background;
      [SerializeField] TextMeshPro _text = null!;
      public TextMeshPro Text => _text;
    }

    public enum RenderingMode
    {
      Default,
      Arena
    }

    public bool IsRevealed => _isRevealed;

    public bool StagingAnimationComplete { get; set; }

    public CardDisplay? Parent { set; get; }

    public SortingOrder? SortingOrder
    {
      set => value?.ApplyTo(_sortingGroup);
    }

    public float ImageWidth => _image.bounds.size.x;

    public void Render(Registry registry, CardView cardView, bool animate = true)
    {
      _registry = registry;
      _cardBack.sprite = _registry.AssetService.GetSprite(cardView.CardBack);
      _outline.sortingOrder = -1;
      _interactive = true;
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
      if (_interactive && _revealedCardView is { Cost: { } cost })
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

        UpdateOutline();
      }
    }

    public void SetRenderingMode(RenderingMode renderingMode)
    {
      if (renderingMode != _renderingMode)
      {
        switch (renderingMode)
        {
          case RenderingMode.Default:
            _frame.gameObject.SetActive(true);
            _titleBackground.gameObject.SetActive(true);
            _title.gameObject.SetActive(true);
            _rulesText.gameObject.SetActive(true);
            _jewel.gameObject.SetActive(true);
            _arenaFrame.gameObject.SetActive(false);
            _cardShadow.SetActive(true);
            _arenaShadow.SetActive(false);
            break;
          case RenderingMode.Arena:
            _frame.gameObject.SetActive(false);
            _titleBackground.gameObject.SetActive(false);
            _title.gameObject.SetActive(false);
            _rulesText.gameObject.SetActive(false);
            _jewel.gameObject.SetActive(false);
            _arenaFrame.gameObject.SetActive(true);
            _cardShadow.SetActive(false);
            _arenaShadow.SetActive(true);
            _topLeftIcon.Background.gameObject.SetActive(false);
            break;
          default:
            Debug.LogError($"Unrecognized rendering mode: {_renderingMode}");
            goto case RenderingMode.Default;
        }
      }

      _renderingMode = renderingMode;
      UpdateIcons();
    }

    void OnMouseDown()
    {
      if (_isDragging)
      {
        // Unity seems to send this event multiple times a lot...
        return;
      }

      if (_interactive && _canPlay)
      {
        _isDragging = true;
        _interactive = false;
        SortingOrder.Create(SortingOrder.Type.Dragging).ApplyTo(_sortingGroup);
        _handIndex = Parent!.RemoveCard(this);
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
      }
    }

    void OnMouseUp()
    {
      if (_isDragging)
      {
        _isDragging = false;
        var distance = _dragStartPosition.z - DragWorldMousePosition().z;
        if (distance < 4.5f || _revealedCardView?.OnReleasePosition == null)
        {
          // Return to hand
          _interactive = true;
          StartCoroutine(Parent!.AddCard(this, animate: true, index: _handIndex));
        }
        else
        {
          StartCoroutine(_registry.CardService.MoveCard(this, _revealedCardView.OnReleasePosition));
        }
      }
    }

    Vector3 DragWorldMousePosition() => _registry.MainCamera.ScreenToWorldPoint(
      new Vector3(Input.mousePosition.x, Input.mousePosition.y, _dragStartScreenZ));

    static void Flip(Component faceUp, Component faceDown, Action onFlipped, bool animate)
    {
      if (animate)
      {
        const float duration = 0.2f;
        DOTween.Sequence()
          .Insert(atPosition: 0, faceDown.transform.DOLocalRotate(new Vector3(x: 0, y: 90, z: 0), duration))
          .InsertCallback(atPosition: duration, () =>
          {
            faceUp.gameObject.SetActive(value: true);
            faceUp.transform.localRotation = Quaternion.Euler(x: 0, y: 90, z: 0);
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
      _frame.sprite = _registry.AssetService.GetSprite(revealed.CardFrame);
      _titleBackground.sprite = _registry.AssetService.GetSprite(revealed.TitleBackground);
      SetTitle(revealed.Title.Text);
      _rulesText.text = revealed.RulesText.Text;
      _jewel.sprite = _registry.AssetService.GetSprite(revealed.Jewel);
      UpdateOutline();
      UpdateIcons();
    }

    void UpdateOutline()
    {
      _outline.gameObject.SetActive(_canPlay);
    }

    void UpdateIcons()
    {
      SetCardIcon(_topLeftIcon, _cardView?.CardIcons?.TopLeftIcon);
      SetCardIcon(_topRightIcon, _cardView?.CardIcons?.TopRightIcon);
      SetCardIcon(_bottomRightIcon, _cardView?.CardIcons?.BottomRightIcon);
      SetCardIcon(_bottomLeftIcon, _cardView?.CardIcons?.BottomLeftIcon);
      SetCardIcon(_centerIcon, _cardView?.CardIcons?.CenterIcon);
    }

    void SetCardIcon(Icon icon, CardIcon? cardIcon)
    {
      if (cardIcon != null && _renderingMode == RenderingMode.Default)
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