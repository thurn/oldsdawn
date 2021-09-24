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

using Spelldawn.Protos;
using UnityEngine;
using static Spelldawn.Masonry.MasonUtil;
using Rotate = UnityEngine.UIElements.Rotate;

#nullable enable

namespace Spelldawn.Battle
{
  public static class RevealedCardNode
  {
    public static Node? Render(RevealedCardView? view, float handPosition)
    {
      if (view is null)
      {
        return null;
      }

      var angle = Mathf.Lerp(-5, 5, handPosition);
      const float cardHeight = 250f;
      const float aspectRatio = 0.6676575505f;
      const float titleBackgroundHeight = 50f;
      const float titleBackgroundAspectRatio = 3.1484375f;
      const float jewelHeight = 12f;
      const float jewelAspectRatio = 0.9312169312f;
      var cardWidth = cardHeight * aspectRatio;

      return Column("Card",
        new FlexStyle
        {
          Margin = TopDip(Mathf.Abs(Mathf.Lerp(-25, 25, handPosition)) + 50f),
          Width = Dip(cardWidth),
          Height = Dip(cardHeight),
          Scale = Scale(0.5f),
          Rotate = Rotate(angle)
        },
        Row(
          "CardImage",
          new FlexStyle
          {
            BackgroundImage = view.Image,
            Position = FlexPosition.Absolute,
            Inset = PositionDip(10f, 16f),
            Width = Dip(144f),
            Height = Dip(144f)
          }),
        Row(
          "CardFrame",
          new FlexStyle
          {
            BackgroundImage = view.CardFrame,
            Width = Dip(cardWidth),
            Height = Dip(cardHeight),
            Position = FlexPosition.Absolute,
            Inset = GroupDip(0)
          }),
        Row(
          "TitleBackground",
          new FlexStyle
          {
            BackgroundImage = view.Webbing,
            Width = Dip(titleBackgroundHeight * titleBackgroundAspectRatio),
            Height = Dip(titleBackgroundHeight),
            Position = FlexPosition.Absolute,
            Inset = PositionDip(4, -12f)
          }),
        Row(
          "Jewel",
          new FlexStyle
          {
            BackgroundImage = view.Jewel,
            Width = Dip(jewelHeight * jewelAspectRatio),
            Height = Dip(jewelHeight),
            Position = FlexPosition.Absolute,
            Inset = PositionDip(78f, 163f)
          }),
        Text(view.Title.Text,
          new FlexStyle
          {
            Position = FlexPosition.Absolute,
            Inset = PositionDip(0, -8f),
            Width = Dip(cardWidth),
            Height = Dip(15f),
            TextAlign = TextAlign.MiddleCenter,
            Color = MakeColor("#4e342e"),
            FontSize = Dip(15f),
            Font = Font("Fonts/Roboto")
          }),
        Text($"<line-height=9>{view.RulesText.Text}</line-height>",
          new FlexStyle
          {
            Position = FlexPosition.Absolute,
            Inset = PositionDip(15f, 178f),
            Width = Dip(130f),
            Height = Dip(55f),
            TextAlign = TextAlign.MiddleCenter,
            Color = MakeColor("#d7ccc8"),
            FontSize = Dip(10f),
            Font = Font("Fonts/Roboto"),
            WhiteSpace = WhiteSpace.Normal,
            ParagraphSpacing = Dip(25f)
          })
      );
    }
  }
}