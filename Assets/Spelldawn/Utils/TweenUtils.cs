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

using DG.Tweening;
using UnityEngine.UIElements;

#nullable enable

namespace Spelldawn.Utils
{
  public static class TweenUtils
  {
    public static Tween ToLeft(VisualElement e, float left, float durationMs) =>
      DOTween.To(
        () => e.style.left.value.value,
        val => e.style.left = val,
        endValue: left,
        duration: durationMs / 1000f);

    public static Tween ToTop(VisualElement e, float top, float durationMs) =>
      DOTween.To(
        () => e.style.top.value.value,
        val => e.style.top = val,
        endValue: top,
        duration: durationMs / 1000f);

    public static Tween ToWidth(VisualElement e, float width, float durationMs) =>
      DOTween.To(
        () => e.style.width.value.value,
        val => e.style.width = val,
        endValue: width,
        duration: durationMs / 1000f);

    public static Tween ToHeight(VisualElement e, float height, float durationMs) =>
      DOTween.To(
        () => e.style.height.value.value,
        val => e.style.height = val,
        endValue: height,
        duration: durationMs / 1000f);
  }
}