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

using static Spelldawn.Masonry.MasonUtil;
using Spelldawn.Masonry;
using Spelldawn.Protos;
using Spelldawn.Services;
using UnityEngine;

#nullable enable

namespace Spelldawn.Battle
{
  public sealed class DeckComponent : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] DebugGroupDip _position;
    [SerializeField] float _size;

    void Update()
    {
    }

    async void Render()
    {
      var element = await Mason.Render(_registry.AssetService, Column("Card",
        new FlexStyle
        {
          Position = FlexPosition.Absolute,
          Width = Dip(_size),
          FixedBackgroundImageAspectRatio = true,
          BackgroundImage = Sprite("LittleSweetDaemon/TCG_Card_Design/Customized/ChampionDeck"),
          Inset = _position.Get()
        }));
      _registry.GameDocument.rootVisualElement.Add(element);
    }
  }
}