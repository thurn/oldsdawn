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
using Color = UnityEngine.Color;

#nullable enable

namespace Spelldawn.Battle
{
  public sealed class CardComponent : MonoBehaviour
  {
    const float CardWidth = 2246f;
    const float WebbingWidth = 2384f;
    const float WebbingHeight = 1566f;

    [SerializeField] Registry _registry = null!;
    CardView _cardView = null!;
    VisualElement? _element;

    void Start()
    {
      _cardView = new CardView
      {
        CardBack = Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Card_Backs/Card_Back_Steampunk_Style_Color_1"),
        CardFrame = Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Cards/Card_Steampunk_Style_Color_1"),
        Webbing = Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Webbings/Webbing_Steampunk_Style_Color_1.png"),
        Jewel = Sprite(
          "Assets/ThirdParty/Resources/LittleSweetDaemon/TCG_Card_Fantasy_Design/Jewels/Jewel_Steampunk_Color_01.png"),
        CardText = new CardText
        {
          Text = "Hello, world!"
        },
        CanPlay = false
      };

      Render();
    }

    async void Render()
    {
      var frame = await _registry.AssetService.LoadSprite(_cardView.CardFrame);

      _element = await Mason.Render(_registry, Column("Frame",
        new FlexStyle
        {
          BackgroundImage = _cardView.CardFrame,
          Position = FlexPosition.Absolute,
          Width = Px(112),
          Height = Px(168),
          Inset = GroupPx(50, 0, 0, 100)
        },
        Row("Square", new FlexStyle
        {
          BackgroundColor = Color(Color.red),
          Width = Px(100),
          Height = Px(100),
          Position = FlexPosition.Absolute,
          Inset = GroupPx(50, 0, 0, 100)
        })
      ));

      _registry.GameDocument.rootVisualElement.Clear();
      _registry.GameDocument.rootVisualElement.Add(_element);
    }
  }
}