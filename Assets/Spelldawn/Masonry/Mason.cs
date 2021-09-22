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
using System.Threading.Tasks;
using Spelldawn.Protos;
using Spelldawn.Services;
using UnityEditor.PackageManager;
using UnityEngine;
using UnityEngine.UIElements;
using Color = UnityEngine.Color;

#nullable enable

namespace Spelldawn.Masonry
{
  public static class Mason
  {
    public static async Task<VisualElement> Render(Registry registry, Node node) => node.NodeCase switch
    {
      Node.NodeOneofCase.Flexbox => await RenderFlexbox(registry, node.Flexbox),
      _ => throw new ArgumentOutOfRangeException()
    };

    public static async Task<VisualElement> RenderFlexbox(Registry registry, Flexbox flexbox)
    {
      var result = new VisualElement();
      return await ApplyStyle(registry, result, flexbox.Style);
    }

    static StyleColor AdaptColor(Protos.Color? color) =>
      color == null ? new StyleColor(StyleKeyword.Null) : new Color(color.Red, color.Green, color.Blue, color.Alpha);

    static async Task<VisualElement> ApplyStyle(Registry registry, VisualElement e, Style style)
    {
      e.style.backgroundColor = AdaptColor(style.BackgroundColor);
      e.style.backgroundImage =
        style.BackgroundImage is { } bi
          ? new StyleBackground(await registry.AssetService.LoadSprite(bi))
          : new StyleBackground(StyleKeyword.Null);
      e.style.height = style.Height?.Value ?? new StyleLength(StyleKeyword.Null);
      e.style.width = style.Width?.Value ?? new StyleLength(StyleKeyword.Null);
      e.style.left = style.Inset?.Left?.Value ?? new StyleLength(StyleKeyword.Null);
      e.style.top = style.Inset?.Top?.Value ?? new StyleLength(StyleKeyword.Null);
      e.style.position = style.Position switch
      {
        FlexPosition.Relative => Position.Relative,
        FlexPosition.Absolute => Position.Absolute,
        _ => new StyleEnum<Position>(StyleKeyword.Null)
      };

      Debug.Log($"ApplyStyle: ");
      // e.style.position = Position.Absolute;
      // e.style.left = 300;
      // e.style.top = 100;
      // e.style.width = 50;
      // e.style.height = 50;
      // e.style.backgroundColor = Color.green;

      return e;
    }
  }
}