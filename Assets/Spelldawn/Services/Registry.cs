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
using System.Threading;
using UnityEngine;
using UnityEngine.UIElements;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class Registry : MonoBehaviour
  {
    public Camera MainCamera => _mainCamera;
    [SerializeField] Camera _mainCamera = null!;

    /// <summary>'Scaled with Screen Size' document for rendering gameplay</summary>
    public UIDocument GameDocument => _gameDocument;
    [SerializeField] UIDocument _gameDocument = null!;

    /// <summary>'Constant Physical Size' document for rendering gameplay</summary>
    public UIDocument InterfaceDocument => _interfaceDocument;
    [SerializeField] UIDocument _interfaceDocument = null!;

    public AssetService AssetService => _assetService;
    [SerializeField] AssetService _assetService = null!;

    void Start()
    {
      var x = 667;
      var y = 375;

      PositionAt(_gameDocument.rootVisualElement, 0, 0);
      PositionAt(_gameDocument.rootVisualElement, x, 0);
      PositionAt(_gameDocument.rootVisualElement, 0, y);
      PositionAt(_gameDocument.rootVisualElement, x, y);
    }

    void PositionAt(VisualElement parent, int x, int y, bool red = false)
    {
      var size = 100;
      parent.Add(new VisualElement
      {
        style =
        {
          position = Position.Absolute,
          left = x - size / 2f,
          top = y - size / 2f,
          width = size,
          height = size,
          backgroundColor = red ? Color.red : Color.yellow
        }
      });

      size = 50;
      parent.Add(new VisualElement
      {
        style =
        {
          position = Position.Absolute,
          left = x - size / 2f,
          top = y - size / 2f,
          width = size,
          height = size,
          backgroundColor = Color.green
        }
      });

      size = 25;
      parent.Add(new VisualElement
      {
        style =
        {
          position = Position.Absolute,
          left = x - size / 2f,
          top = y - size / 2f,
          width = size,
          height = size,
          backgroundColor = Color.blue
        }
      });
    }
  }
}