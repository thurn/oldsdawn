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
using Spelldawn.Protos;
using Spelldawn.Services;
using UnityEngine;
using UnityEngine.UIElements;
using Color = Spelldawn.Protos.Color;
using Length = Spelldawn.Protos.Length;

#nullable enable

namespace Spelldawn.Battle
{
  public sealed class CardComponent : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    CardView _cardView = null!;

    void Start()
    {
      _cardView = new CardView
      {
        CardBack = AssetService.Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Card_Backs/Card_Back_Steampunk_Style_Color_1"),
        CardFrame = AssetService.Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Cards/Card_Steampunk_Style_Color_1"),
        Webbing = AssetService.Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Webbings/Webbing_Steampunk_Style_Color_1.png"),
        Jewel = AssetService.Sprite(
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
      var rendered = await Mason.RenderFlexbox(_registry, new Flexbox
      {
        Style = new Style
        {
          BackgroundImage = _cardView.CardFrame,
          // BackgroundColor = new Color
          // {
          //   Red = 1.0f,
          //   Alpha = 1.0f
          // },
          Position = FlexPosition.Absolute,
          Width = Length(112),
          Height = Length(168),
          Inset = new LengthGroup
          {
            Top = Length(50),
            Left = Length(200)
          }
        }
      });

      _registry.GameDocument.rootVisualElement.Add(rendered);
    }

    static Length Length(float value) => new Length
    {
      Value = value
    };
  }
}