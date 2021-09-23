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
using System.Linq;
using Spelldawn.Protos;
using UnityEngine;
using Node = Spelldawn.Protos.Node;

#nullable enable

namespace Spelldawn.Masonry
{
  public static class MasonUtil
  {
    const float ReferenceDpi = 155f;

    public static Dimension Dip(float value) => new()
    {
      Unit = DimensionUnit.Dip,
      Value = value
    };

    public static Dimension Percent(float value) => new()
    {
      Unit = DimensionUnit.Percentage,
      Value = value
    };

    public static Dimension VMin(float value) => new()
    {
      Unit = DimensionUnit.Vmin,
      Value = value
    };

    public static DimensionGroup LeftTopDip(float left, float top) => GroupDip(top, 0, 0, left);

    public static DimensionGroup GroupDip(float all) => GroupDip(all, all);

    public static DimensionGroup GroupDip(float topBottom, float leftRight) =>
      GroupDip(topBottom, leftRight, topBottom, leftRight);

    public static DimensionGroup GroupDip(float top, float right, float bottom, float left) => new()
    {
      Top = Dip(top),
      Right = Dip(right),
      Bottom = Dip(bottom),
      Left = Dip(left)
    };

    public static DimensionGroup GroupVMin(float all) => GroupVMin(all, all);

    public static DimensionGroup GroupVMin(float topBottom, float leftRight) =>
      GroupVMin(topBottom, leftRight, topBottom, leftRight);

    public static DimensionGroup GroupVMin(float top, float right, float bottom, float left) => new()
    {
      Top = VMin(top),
      Right = VMin(right),
      Bottom = VMin(bottom),
      Left = VMin(left)
    };

    public static FlexColor MakeColor(string hexString)
    {
      if (ColorUtility.TryParseHtmlString(hexString, out var color))
      {
        return MakeColor(color);
      }
      else
      {
        throw new ArgumentException($"Invalid color: {hexString}");
      }
    }

    public static FlexColor MakeColor(Color color) => new()
    {
      Red = color.r,
      Green = color.g,
      Blue = color.b,
      Alpha = color.a
    };

    public static SpriteAddress Sprite(string address) => new()
    {
      Address = address
    };

    public static FontAddress Font(string address) => new()
    {
      Address = address
    };

    public static Node Row(string name, FlexStyle style, params Node?[] children)
    {
      style.FlexDirection = FlexDirection.Row;
      return MakeFlexbox(name, style, children);
    }

    public static Node Column(string name, FlexStyle style, params Node?[] children)
    {
      style.FlexDirection = FlexDirection.Column;
      return MakeFlexbox(name, style, children);
    }

    public static Node Text(string label, FlexStyle style) => new()
    {
      Text = new Text
      {
        Label = label,
        Style = style
      }
    };

    public static float ScreenPxToDip(float value) => value * ReferenceDpi / Screen.dpi;

    public static float VMinToScreenPx(float vmin) => Screen.width < Screen.height
      ? vmin * Screen.width / 100f
      : vmin * Screen.height / 100f;

    public static float VMinToDip(float vmin) => ScreenPxToDip(VMinToScreenPx(vmin));

    /// <summary>
    /// Given a value in units of screen pixels, returns a ratio x such that screenPixels * x = targetVMin
    /// </summary>
    public static float MultiplerForTargetVMin(float targetVMin, float screenPixels) =>
      VMinToScreenPx(targetVMin) / screenPixels;

    /// <summary>
    /// Given a value in units of screen pixels, returns a ratio x such that screenPixels * x = targetDips
    /// </summary>
    public static float MultiplerForTargetDip(float targetDips, float screenPixels) =>
      targetDips / ScreenPxToDip(screenPixels);

    public static FlexScale Scale(float amount) => Scale(amount, amount);

    public static FlexScale Scale(float x, float y) => new()
    {
      Amount = new FlexVector3
      {
        X = x,
        Y = y,
        Z = 0
      }
    };

    static Node MakeFlexbox(string name, FlexStyle style, params Node?[] children)
    {
      var flexbox = new Flexbox
      {
        Style = style,
        Name = name
      };

      flexbox.Children.AddRange(children.Where(child => child != null));

      return new Node
      {
        Flexbox = flexbox
      };
    }
  }
}