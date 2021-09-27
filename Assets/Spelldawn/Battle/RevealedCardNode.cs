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

#nullable enable

namespace Spelldawn.Battle
{
  public static class RevealedCardNode
  {
    public const float ReferenceCardHeight = 250f;
    public const float CardAspectRatio = 0.6676575505f;

    public static Node? Render(RevealedCardView? view, CardProps props)
    {
      if (view is null)
      {
        return null;
      }

      var angle = props.HandPosition.HasValue ? Mathf.Lerp(-5, 5, props.HandPosition.Value) : 0;
      const float titleBackgroundHeight = 50f;
      const float titleBackgroundAspectRatio = 3.1484375f;
      const float jewelHeight = 12f;
      const float jewelAspectRatio = 0.9312169312f;
      const float cardWidth = ReferenceCardHeight * CardAspectRatio;
      var tintColor = MakeColor(Color.gray);

      var result = Column($"Card{props.Id.Value}",
        new FlexStyle
        {
          Width = Dip(cardWidth * props.Scale),
          Height = Dip(ReferenceCardHeight * props.Scale),
          FlexShrink = 0,
          Rotate = Rotate(angle),
          TransitionProperties = { "all" },
          TransitionDurations = { DurationMs(1000) }
        },
        view.ImageBackground is null
          ? null
          : Row(
            "CardImageBackground",
            new FlexStyle
            {
              BackgroundImage = view.ImageBackground,
              BackgroundImageTintColor = props.OverlayDim ? tintColor : null,
              Position = FlexPosition.Absolute,
              Inset = PositionDip(10f * props.Scale, 16f * props.Scale),
              Width = Dip(144f * props.Scale),
              Height = Dip(144f * props.Scale)
            }),
        Row(
          "CardImage",
          new FlexStyle
          {
            BackgroundImage = view.Image,
            BackgroundImageTintColor = props.OverlayDim ? tintColor : null,
            Position = FlexPosition.Absolute,
            Inset = PositionDip(10f * props.Scale, 16f * props.Scale),
            Width = Dip(144f * props.Scale),
            Height = Dip(144f * props.Scale)
          }),
        Row(
          "CardFrame",
          new FlexStyle
          {
            BackgroundImage = view.CardFrame,
            BackgroundImageTintColor = props.OverlayDim ? tintColor : null,
            Width = Dip(cardWidth * props.Scale),
            Height = Dip(ReferenceCardHeight * props.Scale),
            Position = FlexPosition.Absolute,
            Inset = AllDip(0)
          }),
        Row(
          "TitleBackground",
          new FlexStyle
          {
            BackgroundImage = view.TitleBackground,
            BackgroundImageTintColor = props.OverlayDim ? tintColor : null,
            Width = Dip(titleBackgroundHeight * titleBackgroundAspectRatio * props.Scale),
            Height = Dip(titleBackgroundHeight * props.Scale),
            Position = FlexPosition.Absolute,
            Inset = PositionDip(4 * props.Scale, -12f * props.Scale)
          }),
        Row(
          "Jewel",
          new FlexStyle
          {
            BackgroundImage = view.Jewel,
            BackgroundImageTintColor = props.OverlayDim ? tintColor : null,
            Width = Dip(jewelHeight * jewelAspectRatio * props.Scale),
            Height = Dip(jewelHeight * props.Scale),
            Position = FlexPosition.Absolute,
            Inset = PositionDip(78f * props.Scale, 163f * props.Scale)
          }),
        Text(view.Title.Text,
          new FlexStyle
          {
            Position = FlexPosition.Absolute,
            Inset = PositionDip(0, -8f * props.Scale),
            Width = Dip(cardWidth * props.Scale),
            Height = Dip(22f * props.Scale),
            TextAlign = TextAlign.MiddleCenter,
            Color = MakeColor("#4e342e"),
            FontSize = Dip(15f * props.Scale),
            Font = Font("Fonts/Roboto"),
            Padding = AllDip(0),
            Margin = AllDip(0)
          }),
        Text($"<line-height={9 * props.Scale}>{view.RulesText.Text}</line-height>",
          new FlexStyle
          {
            Position = FlexPosition.Absolute,
            Inset = PositionDip(18f * props.Scale, 180f * props.Scale),
            Width = Dip(130f * props.Scale),
            Height = Dip(55f * props.Scale),
            TextAlign = TextAlign.MiddleCenter,
            Color = MakeColor("#d7ccc8"),
            FontSize = Dip(10f * props.Scale),
            Font = Font("Fonts/Roboto"),
            WhiteSpace = WhiteSpace.Normal,
            ParagraphSpacing = Dip(25f * props.Scale),
            Padding = AllDip(0),
            Margin = AllDip(0)
          })
      );

      return Row("CardWrapper",
        new FlexStyle
        {
          Margin = props.HandPosition.HasValue
            ? TopDip(Mathf.Abs(Mathf.Lerp(-25, 25, props.HandPosition.Value)) + 50f)
            : AllDip(0),
          JustifyContent = FlexJustify.Center,
          TransitionProperties = { "all" },
          TransitionDurations = { DurationMs(1000) }
        },
        result);
    }
  }
}