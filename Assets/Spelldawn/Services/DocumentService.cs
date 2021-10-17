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

using Spelldawn.Masonry;
using static Spelldawn.Masonry.MasonUtil;
using Spelldawn.Protos;
using UnityEngine;
using UnityEngine.UIElements;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class DocumentService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;

    [SerializeField] UIDocument _document = null!;

    public void DebugShowRegions()
    {
      _document.rootVisualElement.Clear();
    }

    public void HandleRenderInterface(RenderInterfaceCommand command)
    {
      _document.rootVisualElement.Clear();
      _document.rootVisualElement.Add(Mason.Render(_registry, Elements(command.Node)));
    }

    static Node Elements(Node raidControls) => Column("Root",
      new FlexStyle
      {
        Position = FlexPosition.Absolute,
        Inset = GroupDip(0, 0, 0, 0)
      },
      Row("RaidControls", new FlexStyle
      {
        Position = FlexPosition.Absolute,
        Height = Dip(125),
        Inset = new DimensionGroup
        {
          Left = Dip(0),
          Right = Dip(0),
          Bottom = Dip(160)
        }
      }, raidControls)
    );
  }
}