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

using Spelldawn.Masonry;
using static Spelldawn.Masonry.MasonUtil;
using Spelldawn.Protos;
using Spelldawn.Services;
using UnityEngine;
using UnityEngine.UIElements;

#nullable enable

namespace Spelldawn.Battle
{
  public sealed class CardComponent : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] DebugGroupDip _imagePosition = null!;
    [SerializeField] float _imageWidth;
    [SerializeField] float _imageHeight;
    CardView _cardView = null!;
    VisualElement? _element;

    void Start()
    {
      _cardView = new CardView
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
          Text = "Boom!"
        },
        RulesText = new RulesText
        {
          Text = "Hello, world!"
        },
        CanPlay = false
      };
    }

    void Update()
    {
      Render();
    }

    async void Render()
    {
      var sprite = await _registry.AssetService.LoadSprite(_cardView.CardBack);
      var rect = sprite.value.sprite.rect;
      var imageScale = MultiplerForTargetDip(100, rect.height);

      _element = await Mason.Render(_registry, Column("Card",
        new FlexStyle
        {
          BackgroundImageScaleMultiplier = imageScale,
          Position = FlexPosition.Absolute,
          Width = Dip(100 * rect.width / rect.height),
          Height = Dip(100),
          Scale = Scale(3f),
          Inset = LeftTopDip(150f, 150f),
        },
        Row(
          "CardImage",
          new FlexStyle
          {
            BackgroundImage = _cardView.Image,
            Position = FlexPosition.Absolute,
            Inset = LeftTopDip(4f, 7f),
            Width = Dip(57f),
            Height = Dip(57f)
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
            Inset = LeftTopDip(-2.15f, -2f)
          }),
        Row(
          "Jewel",
          new FlexStyle
          {
            BackgroundImage = _cardView.Jewel,
            BackgroundImageScaleMultiplier = imageScale,
            Position = FlexPosition.Absolute,
            Inset = LeftTopDip(31f, 65f)
          })
      ));

      _registry.Document.rootVisualElement.Clear();
      _registry.Document.rootVisualElement.Add(_element);
    }
  }
}