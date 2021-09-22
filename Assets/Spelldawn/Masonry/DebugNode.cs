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
using Spelldawn.Services;
using UnityEngine;
using UnityEngine.UIElements;

#nullable enable

namespace Spelldawn.Masonry
{
  [Serializable]
  public sealed class DebugDimensionGroup
  {
    public float Top;
    public float Right;
    public float Bottom;
    public float Left;
  }

  public sealed class DebugNode : MonoBehaviour
  {
    [SerializeField] Align _alignItems;
    [SerializeField] Align _alignSelf;
    [SerializeField] Color _backgroundColor;
    [SerializeField] Sprite _backgroundSprite;
    [SerializeField] Justify _justifyContent;
    [SerializeField] DebugDimensionGroup _margin;
    [SerializeField] DebugDimensionGroup _inset;
    [SerializeField] float _width;
    [SerializeField] float _height;

    Registry _registry = null!;
    VisualElement _element = null!;

    public void Initialize(Registry registry, VisualElement element)
    {
      _registry = registry;
      _element = element;
    }

    void Update()
    {
      _alignItems = _element.style.alignItems.value;
      _alignSelf = _element.style.alignSelf.value;
      _backgroundColor = _element.style.backgroundColor.value;
      _backgroundSprite = _element.style.backgroundImage.value.sprite;
      _justifyContent = _element.style.justifyContent.value;
      _margin.Top = _element.style.top.value.value;
      _margin.Right = _element.style.right.value.value;
      _margin.Bottom = _element.style.bottom.value.value;
      _margin.Left = _element.style.left.value.value;
    }
  }
}