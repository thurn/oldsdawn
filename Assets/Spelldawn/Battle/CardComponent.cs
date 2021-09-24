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

using System.Collections.Generic;
using Spelldawn.Masonry;
using Spelldawn.Protos;
using Spelldawn.Services;
using UnityEngine;
using UnityEngine.UIElements;
using static Spelldawn.Masonry.MasonUtil;
using TextShadow = Spelldawn.Protos.TextShadow;
using TimeValue = UnityEngine.UIElements.TimeValue;
using WhiteSpace = Spelldawn.Protos.WhiteSpace;

#nullable enable

namespace Spelldawn.Battle
{
  public sealed class CardComponent : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    RevealedCardView _cardView = null!;
    VisualElement? _element;

    void Start()
    {
      _cardView = new RevealedCardView
      {
        CardBack = Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Backs/Back_Steampunk_Style_Color_1"),
        CardFrame = Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Cards/Card_Steampunk_Style_Color_1"),
        Webbing = Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Webbings/Webbing_Steampunk_Style_Color_1"),
        Jewel = Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Jewels/Jewel_Steampunk_Color_01"),
        Image = Sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_06"),
        Title = new CardTitle
        {
          Text = "Secrets of the Key"
        },
        RulesText = new RulesText
        {
          Text = "Text"
        },
        CanPlay = false
      };

      // Render();
    }

    void Update()
    {
      if (Input.GetMouseButtonDown(0))
      {
        _element!.style.transitionDuration = new StyleList<TimeValue>(
          new List<TimeValue> { 0.5f });
        _element!.style.transitionProperty = new StyleList<StylePropertyName>(
          new List<StylePropertyName> { "scale" });
        _element.style.scale = new StyleScale(new Scale(new Vector3(0f, 2f, 1f)));
        _element.style.transformOrigin =
          new StyleTransformOrigin(new TransformOrigin(Length.Percent(50), Length.Percent(50), 0));
      }
    }

    async void Render()
    {
      var sprite = await _registry.AssetService.LoadSprite(_cardView.CardBack);
      var rect = sprite.value.sprite.rect;
      var imageScale = MultiplerForTargetDip(100, rect.height);
      var cardWidth = Dip(100 * rect.width / rect.height);
      Debug.Log($"Render: {imageScale}");
      Debug.Log($"Render: {cardWidth}");

      _element = await Mason.Render(_registry.AssetService, Column("Card",
        new FlexStyle
        {
          BackgroundImageScaleMultiplier = imageScale,
          Position = FlexPosition.Absolute,
          Width = cardWidth,
          Height = Dip(100),
          Scale = Scale(2f),
          Inset = PositionDip(150f, 125f)
        },
        Row(
          "CardImage",
          new FlexStyle
          {
            BackgroundImage = _cardView.Image,
            Position = FlexPosition.Absolute,
            Inset = PositionDip(4f, 6.5f),
            Width = Dip(58f),
            Height = Dip(58f)
          }),
        Row(
          "CardFrame",
          new FlexStyle
          {
            BackgroundImage = _cardView.CardFrame,
            BackgroundImageScaleMultiplier = imageScale,
            Position = FlexPosition.Absolute,
            Inset = GroupDip(0)
          }),
        Row(
          "Webbing",
          new FlexStyle
          {
            BackgroundImage = _cardView.Webbing,
            BackgroundImageScaleMultiplier = imageScale,
            Position = FlexPosition.Absolute,
            Inset = PositionDip(-2.15f, -4f)
          }),
        Row(
          "Jewel",
          new FlexStyle
          {
            BackgroundImage = _cardView.Jewel,
            BackgroundImageScaleMultiplier = imageScale,
            Position = FlexPosition.Absolute,
            Inset = PositionDip(31f, 65f)
          }),
        Text(_cardView.Title.Text,
          new FlexStyle
          {
            Position = FlexPosition.Absolute,
            Inset = PositionDip(0, -7.3f),
            Width = cardWidth,
            Height = Dip(0f),
            TextAlign = TextAlign.MiddleCenter,
            Color = MakeColor(Color.white),
            TextOutlineColor = MakeColor(Color.black),
            TextOutlineWidth = 0.1f,
            FontSize = Dip(5f),
            Font = Font("Fonts/Impact"),
            TextShadow = new TextShadow
            {
              Color = MakeColor(Color.black),
              Offset = new FlexVector2
              {
                X = 0.1f,
                Y = 0.1f
              },
              BlurRadius = 0.5f
            }
          }),
        Text($"<line-height=4>{_cardView.RulesText}</line-height>",
          new FlexStyle
          {
            Position = FlexPosition.Absolute,
            Inset = PositionDip(4.9f, 67f),
            Width = Dip(53f),
            Height = Dip(25f),
            TextAlign = TextAlign.MiddleCenter,
            Color = MakeColor("#d7ccc8"),
            FontSize = Dip(4f),
            Font = Font("Fonts/Roboto"),
            WhiteSpace = WhiteSpace.Normal,
            ParagraphSpacing = Dip(25f)
          })
      ));

      _registry.GameDocument.rootVisualElement.Add(_element);
    }
  }
}