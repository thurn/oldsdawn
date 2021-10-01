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
using Spelldawn.Protos;
using Spelldawn.Services;
using UnityEngine;
using UnityEngine.UIElements;
using EasingMode = Spelldawn.Protos.EasingMode;
using FlexDirection = Spelldawn.Protos.FlexDirection;
using FontStyle = Spelldawn.Protos.FontStyle;
using OverflowClipBox = Spelldawn.Protos.OverflowClipBox;
using TextOverflow = Spelldawn.Protos.TextOverflow;
using TextOverflowPosition = Spelldawn.Protos.TextOverflowPosition;
using TextShadow = UnityEngine.UIElements.TextShadow;
using TimeValue = UnityEngine.UIElements.TimeValue;
using WhiteSpace = Spelldawn.Protos.WhiteSpace;

#nullable enable

namespace Spelldawn.Masonry
{
  public static class Mason
  {
    /// <summary>
    /// Renders the provided Node into a VisualElement.
    /// </summary>
    public static VisualElement Render(Registry registry, Node node)
    {
      var element = CreateElement(node);
      return ApplyToElement(registry, node, element);
    }

    public static VisualElement CreateElement(Node node) => node.NodeType?.TypeCase switch
    {
      NodeType.TypeOneofCase.Text => new NodeLabel(),
      _ => new NodeVisualElement()
    };

    public static VisualElement ApplyToElement(Registry registry, Node node, VisualElement element)
    {
      switch (node.NodeType?.TypeCase)
      {
        case NodeType.TypeOneofCase.Text:
          ApplyText(node.NodeType.Text, (NodeLabel)element);
          break;
      }

      return ApplyNode(registry, node, element);
    }

    static VisualElement ApplyNode(Registry registry, Node node, VisualElement element)
    {
      element.name = node.Name;

      foreach (var child in node.Children)
      {
        element.Add(Render(registry, child));
      }

      var result = ApplyStyle(registry, element, node.Style);
      var callbacks = ((INodeCallbacks)element);

      if (node.HoverStyle != null)
      {
        var hoverStyle = new FlexStyle();
        hoverStyle.MergeFrom(node.Style);
        hoverStyle.MergeFrom(node.HoverStyle);
        callbacks.SetCallback(new EventCallback<MouseEnterEvent>(_ =>
        {
          ApplyStyle(registry, element, hoverStyle);
        }));
        callbacks.SetCallback(new EventCallback<MouseLeaveEvent>(_ =>
        {
          ApplyStyle(registry, element, node.Style);
        }));
      }
      else
      {
        callbacks.SetCallback<MouseEnterEvent>(null);
        callbacks.SetCallback<MouseLeaveEvent>(null);
      }

      if (node.PressedStyle != null)
      {
        var pressedStyle = new FlexStyle();
        pressedStyle.MergeFrom(node.Style);
        pressedStyle.MergeFrom(node.PressedStyle);
        callbacks.SetCallback(new EventCallback<MouseDownEvent>(_ =>
        {
          ApplyStyle(registry, element, pressedStyle);
        }));
        callbacks.SetCallback(new EventCallback<MouseUpEvent>(_ =>
        {
          var style = node.Style;
          if (node.HoverStyle != null)
          {
            style = new FlexStyle();
            style.MergeFrom(node.Style);
            style.MergeFrom(node.HoverStyle);
          }

          ApplyStyle(registry, element, style);
        }));
      }
      else
      {
        callbacks.SetCallback<MouseDownEvent>(null);
        callbacks.SetCallback<MouseUpEvent>(null);
      }

      if (node.EventHandlers?.ClickAction is {} clickAction)
      {
        callbacks.SetCallback(new EventCallback<ClickEvent>(_ =>
        {
          registry.ActionService.HandleAction(clickAction);
        }));
      }
      else
      {
        callbacks.SetCallback<ClickEvent>(null);
      }

      return result;
    }

    static void ApplyText(Text text, Label label)
    {
      label.text = text.Label;
    }

    static Color AdaptColorNonNull(FlexColor color) =>
      new(color.Red, color.Green, color.Blue, color.Alpha);

    static StyleColor AdaptColor(FlexColor? color) =>
      color == null ? new StyleColor(StyleKeyword.Null) : AdaptColorNonNull(color);

    static StyleFloat AdaptFloat(float? input) => input ?? new StyleFloat(StyleKeyword.Null);

    static StyleInt AdaptInt(int? input) => input ?? new StyleInt(StyleKeyword.Null);

    static Vector2 AdaptVector2(FlexVector2? input) => input is { } v ? new Vector2(v.X, v.Y) : Vector2.zero;

    static Vector3 AdaptVector3(FlexVector3? input) => input is { } v ? new Vector3(v.X, v.Y, v.Z) : Vector2.zero;

    static Length AdaptDimensionNonNull(Dimension dimension, float multiplier = 1f) => dimension.Unit switch
    {
      DimensionUnit.Dip => new Length(dimension.Value * multiplier),
      DimensionUnit.Percentage => Length.Percent(dimension.Value * multiplier),
      DimensionUnit.Vmin => new Length(MasonUtil.VMinToDip(dimension.Value * multiplier)),
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

    public static VisualElement ApplyStyle(Registry registry, VisualElement e, FlexStyle? input)
    {
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
        FlexDirection.Column => UnityEngine.UIElements.FlexDirection.Column,
        FlexDirection.ColumnReverse => UnityEngine.UIElements.FlexDirection.ColumnReverse,
        FlexDirection.Row => UnityEngine.UIElements.FlexDirection.Row,
        FlexDirection.RowReverse => UnityEngine.UIElements.FlexDirection.RowReverse,
        _ => new StyleEnum<UnityEngine.UIElements.FlexDirection>(StyleKeyword.Null)
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
        TextOverflow.Clip => UnityEngine.UIElements.TextOverflow.Clip,
        TextOverflow.Ellipsis => UnityEngine.UIElements.TextOverflow.Ellipsis,
        _ => new StyleEnum<UnityEngine.UIElements.TextOverflow>(StyleKeyword.Null)
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
        EasingMode.Ease => UnityEngine.UIElements.EasingMode.Ease,
        EasingMode.EaseIn => UnityEngine.UIElements.EasingMode.EaseIn,
        EasingMode.EaseOut => UnityEngine.UIElements.EasingMode.EaseOut,
        EasingMode.EaseInOut => UnityEngine.UIElements.EasingMode.EaseInOut,
        EasingMode.Linear => UnityEngine.UIElements.EasingMode.Linear,
        EasingMode.EaseInSine => UnityEngine.UIElements.EasingMode.EaseInSine,
        EasingMode.EaseOutSine => UnityEngine.UIElements.EasingMode.EaseOutSine,
        EasingMode.EaseInOutSine => UnityEngine.UIElements.EasingMode.EaseInOutSine,
        EasingMode.EaseInCubic => UnityEngine.UIElements.EasingMode.EaseInCubic,
        EasingMode.EaseOutCubic => UnityEngine.UIElements.EasingMode.EaseOutCubic,
        EasingMode.EaseInOutCubic => UnityEngine.UIElements.EasingMode.EaseInOutCubic,
        EasingMode.EaseInCirc => UnityEngine.UIElements.EasingMode.EaseInCirc,
        EasingMode.EaseOutCirc => UnityEngine.UIElements.EasingMode.EaseOutCirc,
        EasingMode.EaseInOutCirc => UnityEngine.UIElements.EasingMode.EaseInOutCirc,
        EasingMode.EaseInElastic => UnityEngine.UIElements.EasingMode.EaseInElastic,
        EasingMode.EaseOutElastic => UnityEngine.UIElements.EasingMode.EaseOutElastic,
        EasingMode.EaseInOutElastic => UnityEngine.UIElements.EasingMode.EaseInOutElastic,
        EasingMode.EaseInBack => UnityEngine.UIElements.EasingMode.EaseInBack,
        EasingMode.EaseOutBack => UnityEngine.UIElements.EasingMode.EaseOutBack,
        EasingMode.EaseInOutBack => UnityEngine.UIElements.EasingMode.EaseInOutBack,
        EasingMode.EaseInBounce => UnityEngine.UIElements.EasingMode.EaseInBounce,
        EasingMode.EaseOutBounce => UnityEngine.UIElements.EasingMode.EaseOutBounce,
        EasingMode.EaseInOutBounce => UnityEngine.UIElements.EasingMode.EaseInOutBounce,
        _ => UnityEngine.UIElements.EasingMode.Ease
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
        ? new StyleFontDefinition(registry.AssetService.GetFont(font))
        : new StyleFontDefinition(StyleKeyword.Null);
      e.style.unityFontStyleAndWeight = input.FontStyle switch
      {
        FontStyle.Normal => UnityEngine.FontStyle.Normal,
        FontStyle.Bold => UnityEngine.FontStyle.Bold,
        FontStyle.Italic => UnityEngine.FontStyle.Italic,
        FontStyle.BoldAndItalic => UnityEngine.FontStyle.BoldAndItalic,
        _ => new StyleEnum<UnityEngine.FontStyle>(StyleKeyword.Null)
      };
      e.style.unityOverflowClipBox = input.OverflowClipBox switch
      {
        OverflowClipBox.PaddingBox => UnityEngine.UIElements.OverflowClipBox.PaddingBox,
        OverflowClipBox.ContentBox => UnityEngine.UIElements.OverflowClipBox.ContentBox,
        _ => new StyleEnum<UnityEngine.UIElements.OverflowClipBox>(StyleKeyword.Null)
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
        TextOverflowPosition.End => UnityEngine.UIElements.TextOverflowPosition.End,
        TextOverflowPosition.Start => UnityEngine.UIElements.TextOverflowPosition.Start,
        TextOverflowPosition.Middle => UnityEngine.UIElements.TextOverflowPosition.Middle,
        _ => new StyleEnum<UnityEngine.UIElements.TextOverflowPosition>(StyleKeyword.Null)
      };
      e.style.visibility = input.Visibility switch
      {
        FlexVisibility.Visible => Visibility.Visible,
        FlexVisibility.Hidden => Visibility.Hidden,
        _ => new StyleEnum<Visibility>(StyleKeyword.Null)
      };
      e.style.whiteSpace = input.WhiteSpace switch
      {
        WhiteSpace.Normal => UnityEngine.UIElements.WhiteSpace.Normal,
        WhiteSpace.NoWrap => UnityEngine.UIElements.WhiteSpace.NoWrap,
        _ => new StyleEnum<UnityEngine.UIElements.WhiteSpace>(StyleKeyword.Null)
      };
      e.style.width = AdaptDimension(input.Width);
      e.style.wordSpacing = AdaptDimension(input.WordSpacing);

      if (input.BackgroundImage is { } bi)
      {
        var sprite = registry.AssetService.GetSprite(bi);
        e.style.backgroundImage = new StyleBackground(sprite);

        if (input.BackgroundImageScaleMultiplier is { } multiplier && sprite && sprite != null)
        {
          e.style.width = MasonUtil.ScreenPxToDip(sprite.rect.width * multiplier);
          e.style.height = MasonUtil.ScreenPxToDip(sprite.rect.height * multiplier);
        }

        switch (input.FixedBackgroundImageAspectRatio)
        {
          case true when input.Width is { } width && sprite && sprite != null:
            e.style.height = AdaptDimensionNonNull(width, sprite.rect.height / sprite.rect.width);
            break;
          case true when input.Height is { } height && sprite && sprite != null:
            e.style.width = AdaptDimensionNonNull(height, sprite.rect.width / sprite.rect.height);
            break;
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