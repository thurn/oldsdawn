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

using UnityEngine;
using UnityEngine.UIElements;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class Registry : MonoBehaviour
  {
    [SerializeField] Camera _mainCamera = null!;
    public Camera MainCamera => _mainCamera;

    [SerializeField] UIDocument _document = null!;
    public UIDocument Document => _document;
    public VisualElement RootElement => _document.rootVisualElement;

    void Start()
    {
      var x = 781;
      var y = 360;
      PositionAt(RootElement, 0, 0);
      PositionAt(RootElement, x, 0);
      PositionAt(RootElement, 0, y);
      PositionAt(RootElement, x, y);
    }

    void PositionAt(VisualElement parent, int x, int y)
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
          backgroundColor = Color.red
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