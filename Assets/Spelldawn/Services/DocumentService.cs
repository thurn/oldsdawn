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

using Spelldawn.Battle;
using Spelldawn.Masonry;
using Spelldawn.Utils;
using UnityEngine;
using UnityEngine.UIElements;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class DocumentService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;

    void Start()
    {
      Render();
    }

    public VisualElement Find(string elementName) =>
      Errors.CheckNotNull(_registry.GameDocument.rootVisualElement.Q(elementName), $"Name not found: {elementName}");

    async void Render()
    {
      var node = GameNode.Render(SampleData.SampleView());
      if (node != null)
      {
        var element = await Mason.Render(_registry, node);
        _registry.GameDocument.rootVisualElement.Add(element);
      }
    }
  }
}