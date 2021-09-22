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

using System.Collections.Generic;
using System.Threading.Tasks;
using Cysharp.Threading.Tasks;
using Spelldawn.Protos;
using UnityEngine;
using UnityEngine.UIElements;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class AssetService : MonoBehaviour
  {
    readonly Dictionary<string, StyleBackground> _sprites = new();
    readonly Dictionary<string, StyleFontDefinition> _fonts = new();

    public async UniTask<StyleBackground> LoadSprite(SpriteAddress spriteAddress)
    {
      if (_sprites.ContainsKey(spriteAddress.Address))
      {
        return _sprites[spriteAddress.Address];
      }

      var result = await Resources.LoadAsync<Sprite>(spriteAddress.Address);
      if (result is Sprite s)
      {
        var background = new StyleBackground(s);
        _sprites[spriteAddress.Address] = background;
        return background;
      }
      else
      {
        Debug.LogError($"Sprite not found: {spriteAddress.Address}");
        return new StyleBackground(StyleKeyword.Null);
      }
    }

    public async UniTask<StyleFontDefinition> LoadFont(FontAddress fontAddress)
    {
      if (_fonts.ContainsKey(fontAddress.Address))
      {
        return _fonts[fontAddress.Address];
      }

      var result = await Resources.LoadAsync<Font>(fontAddress.Address);
      if (result is Font f)
      {
        var font = new StyleFontDefinition(f);
        _fonts[fontAddress.Address] = font;
        return font;
      }
      else
      {
        Debug.LogError($"Font not found: {fontAddress.Address}");
        return new StyleFontDefinition(StyleKeyword.Null);
      }
    }
  }
}