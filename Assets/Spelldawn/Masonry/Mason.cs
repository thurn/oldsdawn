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
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Spelldawn.Protos;
using Spelldawn.Services;
using UnityEngine;
using UnityEngine.UIElements;
using EasingMode = UnityEngine.UIElements.EasingMode;
using FlexDirection = UnityEngine.UIElements.FlexDirection;
using FontStyle = UnityEngine.FontStyle;
using TextOverflow = UnityEngine.UIElements.TextOverflow;
using TextShadow = UnityEngine.UIElements.TextShadow;
using TimeValue = UnityEngine.UIElements.TimeValue;
using OverflowClipBox = UnityEngine.UIElements.OverflowClipBox;
using TextOverflowPosition = UnityEngine.UIElements.TextOverflowPosition;
using WhiteSpace = UnityEngine.UIElements.WhiteSpace;

#nullable enable

namespace Spelldawn.Masonry
{
  public static class Mason
  {
    public static Task<VisualElement?> Render(Registry registry, Node? node)
    {
      registry.CheckIsMainThread();

      return node?.NodeCase switch
      {
        Node.NodeOneofCase.Flexbox => RenderFlexbox(registry, node.Flexbox),
        Node.NodeOneofCase.Text => RenderText(registry, node.Text),
        _ => Task.FromResult<VisualElement?>(null)
      };
    }

    public static async Task<VisualElement?> RenderFlexbox(Registry registry, Flexbox flexbox)
    {
      var result = new VisualElement
      {
        name = flexbox.Name
      };

      var children = await Task.WhenAll(flexbox.Children.Select(node => Render(registry, node)));

      foreach (var child in children)
      {
        result.Add(child);
      }

      return await ApplyStyle(registry, result, flexbox.Style);
    }

    public static async Task<VisualElement?> RenderText(Registry registry, Text text)
    {
      var result = new Label
      {
        text = text.Label
      };

      return await ApplyStyle(registry, result, text.Style);
    }

    static Color AdaptColorNonNull(FlexColor color) =>
      new(color.Red, color.Green, color.Blue, color.Alpha);

    static StyleColor AdaptColor(FlexColor? color) =>
      color == null ? new StyleColor(StyleKeyword.Null) : AdaptColorNonNull(color);

    static StyleFloat AdaptFloat(float? input) => input ?? new StyleFloat(StyleKeyword.Null);

    static StyleInt AdaptInt(int? input) => input ?? new StyleInt(StyleKeyword.Null);

    static Vector2 AdaptVector2(FlexVector2? input) => input is { } v ? new Vector2(v.X, v.Y) : Vector2.zero;

    static Vector3 AdaptVector3(FlexVector3? input) => input is { } v ? new Vector3(v.X, v.Y, v.Z) : Vector2.zero;

    static Length AdaptDimensionNonNull(Dimension dimension) => dimension.Unit switch
    {
      DimensionUnit.Dip => new Length(dimension.Value),
      DimensionUnit.Percentage => Length.Percent(dimension.Value),
      DimensionUnit.Vmin => new Length(MasonUtil.VMinToDip(dimension.Value)),
      _ => new Length()
    };

    static StyleLength AdaptDimension(Dimension? dimension) =>
      dimension is { } d ? AdaptDimensionNonNull(d) : new StyleLength(StyleKeyword.Null);

    static StyleEnum<Align> AdaptAlign(FlexAlign input) => input switch
    {
      FlexAlign.Auto => Align.Auto,
      FlexAlign.FlexStart => Align.FlexStart,
      FlexAlign.Center => Align.Center,
      FlexAlign.FlexEnd => Align.FlexEnd,
      FlexAlign.Stretch => Align.Stretch,
      _ => new StyleEnum<Align>(StyleKeyword.Null)
    };

    static StyleList<TResult> AdaptList<TSource, TResult>(IList<TSource> field, Func<TSource, TResult> selector) =>
      field.Count == 0
        ? new StyleList<TResult>(StyleKeyword.Null)
        : new StyleList<TResult>(field.Select(selector).ToList());

    public static async Task<VisualElement> ApplyStyle(Registry registry, VisualElement e, FlexStyle? input)
    {
      registry.CheckIsMainThread();

      if (input == null)
      {
        return e;
      }

      e.style.alignContent = AdaptAlign(input.AlignContent);
      e.style.alignItems = AdaptAlign(input.AlignItems);
      e.style.alignSelf = AdaptAlign(input.AlignSelf);
      e.style.backgroundColor = AdaptColor(input.BackgroundColor);
      e.style.borderTopColor = AdaptColor(input.BorderColor?.Top);
      e.style.borderRightColor = AdaptColor(input.BorderColor?.Right);
      e.style.borderBottomColor = AdaptColor(input.BorderColor?.Bottom);
      e.style.borderLeftColor = AdaptColor(input.BorderColor?.Left);
      e.style.borderTopLeftRadius = AdaptDimension(input.BorderRadius?.TopLeft);
      e.style.borderTopRightRadius = AdaptDimension(input.BorderRadius?.TopRight);
      e.style.borderBottomRightRadius = AdaptDimension(input.BorderRadius?.BottomRight);
      e.style.borderBottomLeftRadius = AdaptDimension(input.BorderRadius?.BottomLeft);
      e.style.borderTopWidth = AdaptFloat(input.BorderWidth?.Top);
      e.style.borderRightWidth = AdaptFloat(input.BorderWidth?.Right);
      e.style.borderBottomWidth = AdaptFloat(input.BorderWidth?.Bottom);
      e.style.borderLeftWidth = AdaptFloat(input.BorderWidth?.Left);
      e.style.top = AdaptDimension(input.Inset?.Top);
      e.style.right = AdaptDimension(input.Inset?.Right);
      e.style.bottom = AdaptDimension(input.Inset?.Bottom);
      e.style.left = AdaptDimension(input.Inset?.Left);
      e.style.color = AdaptColor(input.Color);
      e.style.display = input.Display switch
      {
        FlexDisplayStyle.Flex => DisplayStyle.Flex,
        FlexDisplayStyle.None => DisplayStyle.None,
        _ => new StyleEnum<DisplayStyle>(StyleKeyword.Null)
      };
      e.style.flexBasis = AdaptDimension(input.FlexBasis);
      e.style.flexDirection = input.FlexDirection switch
      {
        Protos.FlexDirection.Column => FlexDirection.Column,
        Protos.FlexDirection.ColumnReverse => FlexDirection.ColumnReverse,
        Protos.FlexDirection.Row => FlexDirection.Row,
        Protos.FlexDirection.RowReverse => FlexDirection.RowReverse,
        _ => new StyleEnum<FlexDirection>(StyleKeyword.Null)
      };
      e.style.flexGrow = AdaptFloat(input.FlexGrow);
      e.style.flexShrink = AdaptFloat(input.FlexShrink);
      e.style.flexWrap = input.Wrap switch
      {
        FlexWrap.NoWrap => Wrap.NoWrap,
        FlexWrap.Wrap => Wrap.Wrap,
        FlexWrap.WrapReverse => Wrap.WrapReverse,
        _ => new StyleEnum<Wrap>(StyleKeyword.Null)
      };
      e.style.fontSize = AdaptDimension(input.FontSize);
      e.style.height = AdaptDimension(input.Height);
      e.style.justifyContent = input.JustifyContent switch
      {
        FlexJustify.FlexStart => Justify.FlexStart,
        FlexJustify.Center => Justify.Center,
        FlexJustify.FlexEnd => Justify.FlexEnd,
        FlexJustify.SpaceBetween => Justify.SpaceBetween,
        FlexJustify.SpaceAround => Justify.SpaceAround,
        _ => new StyleEnum<Justify>(StyleKeyword.Null)
      };
      e.style.letterSpacing = AdaptDimension(input.LetterSpacing);
      e.style.marginTop = AdaptDimension(input.Margin?.Top);
      e.style.marginRight = AdaptDimension(input.Margin?.Right);
      e.style.marginBottom = AdaptDimension(input.Margin?.Bottom);
      e.style.marginLeft = AdaptDimension(input.Margin?.Left);
      e.style.maxHeight = AdaptDimension(input.MaxHeight);
      e.style.maxWidth = AdaptDimension(input.MaxWidth);
      e.style.minHeight = AdaptDimension(input.MinHeight);
      e.style.minWidth = AdaptDimension(input.MinWidth);
      e.style.opacity = AdaptFloat(input.Opacity);
      e.style.overflow = input.Overflow switch
      {
        FlexOverflow.Visible => Overflow.Visible,
        FlexOverflow.Hidden => Overflow.Hidden,
        _ => new StyleEnum<Overflow>(StyleKeyword.Null)
      };
      e.style.paddingTop = AdaptDimension(input.Padding?.Top);
      e.style.paddingRight = AdaptDimension(input.Padding?.Right);
      e.style.paddingBottom = AdaptDimension(input.Padding?.Bottom);
      e.style.paddingLeft = AdaptDimension(input.Padding?.Left);
      e.style.position = input.Position switch
      {
        FlexPosition.Relative => Position.Relative,
        FlexPosition.Absolute => Position.Absolute,
        _ => new StyleEnum<Position>(StyleKeyword.Null)
      };
      e.style.rotate = input.Rotate is { } r
        ? new Rotate(Angle.Degrees(r.Degrees))
        : new StyleRotate(StyleKeyword.Null);
      e.style.scale = input.Scale is { } s ? new Scale(AdaptVector3(s.Amount)) : new StyleScale(StyleKeyword.Null);
      e.style.textOverflow = input.TextOverflow switch
      {
        Protos.TextOverflow.Clip => TextOverflow.Clip,
        Protos.TextOverflow.Ellipsis => TextOverflow.Ellipsis,
        _ => new StyleEnum<TextOverflow>(StyleKeyword.Null)
      };
      e.style.textShadow = input.TextShadow is { } ts
        ? new TextShadow
        {
          offset = AdaptVector2(ts.Offset),
          blurRadius = ts.BlurRadius,
          color = AdaptColorNonNull(ts.Color)
        }
        : new StyleTextShadow(StyleKeyword.Null);
      e.style.transformOrigin = input.TransformOrigin is { } to
        ? new TransformOrigin(AdaptDimensionNonNull(to.X), AdaptDimensionNonNull(to.Y), to.Z)
        : new StyleTransformOrigin(StyleKeyword.Null);
      e.style.transitionDelay =
        AdaptList(input.TransitionDelays, t => new TimeValue(t.Milliseconds, TimeUnit.Millisecond));
      e.style.transitionDuration = AdaptList(input.TransitionDurations,
        t => new TimeValue(t.Milliseconds, TimeUnit.Millisecond));
      e.style.transitionProperty = AdaptList(input.TransitionProperties, p => new StylePropertyName(p));
      e.style.transitionTimingFunction = AdaptList(input.TransitionEasingModes, mode => new EasingFunction(mode switch
      {
        Protos.EasingMode.Ease => EasingMode.Ease,
        Protos.EasingMode.EaseIn => EasingMode.EaseIn,
        Protos.EasingMode.EaseOut => EasingMode.EaseOut,
        Protos.EasingMode.EaseInOut => EasingMode.EaseInOut,
        Protos.EasingMode.Linear => EasingMode.Linear,
        Protos.EasingMode.EaseInSine => EasingMode.EaseInSine,
        Protos.EasingMode.EaseOutSine => EasingMode.EaseOutSine,
        Protos.EasingMode.EaseInOutSine => EasingMode.EaseInOutSine,
        Protos.EasingMode.EaseInCubic => EasingMode.EaseInCubic,
        Protos.EasingMode.EaseOutCubic => EasingMode.EaseOutCubic,
        Protos.EasingMode.EaseInOutCubic => EasingMode.EaseInOutCubic,
        Protos.EasingMode.EaseInCirc => EasingMode.EaseInCirc,
        Protos.EasingMode.EaseOutCirc => EasingMode.EaseOutCirc,
        Protos.EasingMode.EaseInOutCirc => EasingMode.EaseInOutCirc,
        Protos.EasingMode.EaseInElastic => EasingMode.EaseInElastic,
        Protos.EasingMode.EaseOutElastic => EasingMode.EaseOutElastic,
        Protos.EasingMode.EaseInOutElastic => EasingMode.EaseInOutElastic,
        Protos.EasingMode.EaseInBack => EasingMode.EaseInBack,
        Protos.EasingMode.EaseOutBack => EasingMode.EaseOutBack,
        Protos.EasingMode.EaseInOutBack => EasingMode.EaseInOutBack,
        Protos.EasingMode.EaseInBounce => EasingMode.EaseInBounce,
        Protos.EasingMode.EaseOutBounce => EasingMode.EaseOutBounce,
        Protos.EasingMode.EaseInOutBounce => EasingMode.EaseInOutBounce,
        _ => EasingMode.Ease
      }));
      e.style.translate = input.Translate is { } translate
        ? new Translate(AdaptDimensionNonNull(translate.X), AdaptDimensionNonNull(translate.Y), translate.Z)
        : new StyleTranslate(StyleKeyword.Null);
      e.style.unityBackgroundImageTintColor = AdaptColor(input.BackgroundImageTintColor);
      e.style.unityBackgroundScaleMode = input.BackgroundImageScaleMode switch
      {
        ImageScaleMode.StretchToFill => ScaleMode.StretchToFill,
        ImageScaleMode.ScaleAndCrop => ScaleMode.ScaleAndCrop,
        ImageScaleMode.ScaleToFit => ScaleMode.ScaleToFit,
        _ => new StyleEnum<ScaleMode>(StyleKeyword.Null)
      };
      e.style.unityFontDefinition = input.Font is { } font
        ? await registry.AssetService.LoadFont(font)
        : new StyleFontDefinition(StyleKeyword.Null);
      e.style.unityFontStyleAndWeight = input.FontStyle switch
      {
        Protos.FontStyle.Normal => FontStyle.Normal,
        Protos.FontStyle.Bold => FontStyle.Bold,
        Protos.FontStyle.Italic => FontStyle.Italic,
        Protos.FontStyle.BoldAndItalic => FontStyle.BoldAndItalic,
        _ => new StyleEnum<FontStyle>(StyleKeyword.Null)
      };
      e.style.unityOverflowClipBox = input.OverflowClipBox switch
      {
        Protos.OverflowClipBox.PaddingBox => OverflowClipBox.PaddingBox,
        Protos.OverflowClipBox.ContentBox => OverflowClipBox.ContentBox,
        _ => new StyleEnum<OverflowClipBox>(StyleKeyword.Null)
      };
      e.style.unityParagraphSpacing = AdaptDimension(input.ParagraphSpacing);
      e.style.unitySliceTop = AdaptInt(input.ImageSlice?.Top);
      e.style.unitySliceRight = AdaptInt(input.ImageSlice?.Right);
      e.style.unitySliceBottom = AdaptInt(input.ImageSlice?.Bottom);
      e.style.unitySliceLeft = AdaptInt(input.ImageSlice?.Left);
      e.style.unityTextAlign = input.TextAlign switch
      {
        TextAlign.UpperLeft => TextAnchor.UpperLeft,
        TextAlign.UpperCenter => TextAnchor.UpperCenter,
        TextAlign.UpperRight => TextAnchor.UpperRight,
        TextAlign.MiddleLeft => TextAnchor.MiddleLeft,
        TextAlign.MiddleCenter => TextAnchor.MiddleCenter,
        TextAlign.MiddleRight => TextAnchor.MiddleRight,
        TextAlign.LowerLeft => TextAnchor.LowerLeft,
        TextAlign.LowerCenter => TextAnchor.LowerCenter,
        TextAlign.LowerRight => TextAnchor.LowerRight,
        _ => new StyleEnum<TextAnchor>(StyleKeyword.Null)
      };
      e.style.unityTextOutlineColor = AdaptColor(input.TextOutlineColor);
      e.style.unityTextOutlineWidth = AdaptFloat(input.TextOutlineWidth);
      e.style.unityTextOverflowPosition = input.TextOverflowPosition switch
      {
        Protos.TextOverflowPosition.End => TextOverflowPosition.End,
        Protos.TextOverflowPosition.Start => TextOverflowPosition.Start,
        Protos.TextOverflowPosition.Middle => TextOverflowPosition.Middle,
        _ => new StyleEnum<TextOverflowPosition>(StyleKeyword.Null)
      };
      e.style.visibility = input.Visibility switch
      {
        FlexVisibility.Visible => Visibility.Visible,
        FlexVisibility.Hidden => Visibility.Hidden,
        _ => new StyleEnum<Visibility>(StyleKeyword.Null)
      };
      e.style.whiteSpace = input.WhiteSpace switch
      {
        Protos.WhiteSpace.Normal => WhiteSpace.Normal,
        Protos.WhiteSpace.NoWrap => WhiteSpace.NoWrap,
        _ => new StyleEnum<WhiteSpace>(StyleKeyword.Null)
      };
      e.style.width = AdaptDimension(input.Width);
      e.style.wordSpacing = AdaptDimension(input.WordSpacing);

      if (input.BackgroundImage is { } bi)
      {
        var sprite = await registry.AssetService.LoadSprite(bi);
        e.style.backgroundImage = sprite;

        if (input.BackgroundImageScaleMultiplier is { } multiplier && sprite.value.sprite is { } sp)
        {
          e.style.width = MasonUtil.ScreenPxToDip(sp.rect.width * multiplier);
          e.style.height = MasonUtil.ScreenPxToDip(sp.rect.height * multiplier);
        }
      }
      else
      {
        e.style.backgroundImage = new StyleBackground(StyleKeyword.Null);
      }

      return e;
    }
  }
}