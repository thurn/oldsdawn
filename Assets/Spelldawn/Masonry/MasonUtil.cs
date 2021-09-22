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

using System.Linq;
using Spelldawn.Protos;
using UnityEditor;
using UnityEngine;
using Node = Spelldawn.Protos.Node;

#nullable enable

namespace Spelldawn.Masonry
{
  public static class MasonUtil
  {
    public static Dimension Px(float value) => new()
    {
      Unit = DimensionUnit.Pixel,
      Value = value
    };

    public static Dimension Percent(float value) => new()
    {
      Unit = DimensionUnit.Percentage,
      Value = value
    };

    public static DimensionGroup GroupPx(float all) => GroupPx(all, all);

    public static DimensionGroup GroupPx(float topBottom, float leftRight) =>
      GroupPx(topBottom, leftRight, topBottom, leftRight);

    public static DimensionGroup GroupPx(float top, float right, float bottom, float left) => new()
    {
      Top = Px(top),
      Right = Px(right),
      Bottom = Px(bottom),
      Left = Px(left)
    };

    public static FlexColor Color(Color color) => new()
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