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
using System.Threading.Tasks;
using Spelldawn.Protos;
using UnityEngine;
using UnityEngine.UIElements;
using Object = UnityEngine.Object;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class AssetService : MonoBehaviour
  {
    readonly Dictionary<string, StyleBackground> _sprites = new();
    readonly Dictionary<string, StyleFontDefinition> _fonts = new();

    public Task<StyleBackground> LoadSprite(SpriteAddress spriteAddress)
    {
      if (_sprites.ContainsKey(spriteAddress.Address))
      {
        return Task.FromResult(_sprites[spriteAddress.Address]);
      }

      TaskCompletionSource<StyleBackground> result = new();
      StartCoroutine(LoadResourceAsync<Sprite>(spriteAddress.Address, s =>
      {
        if (s == null)
        {
          Debug.LogError($"Sprite not found: {spriteAddress.Address}");
          result.SetResult(new StyleBackground(StyleKeyword.Null));
        }
        else
        {
          var background = new StyleBackground(s);
          _sprites[spriteAddress.Address] = background;
          result.SetResult(background);
        }
      }));

      return result.Task;
    }

    public Task<StyleFontDefinition> LoadFont(FontAddress fontAddress)
    {
      if (_fonts.ContainsKey(fontAddress.Address))
      {
        return Task.FromResult(_fonts[fontAddress.Address]);
      }

      TaskCompletionSource<StyleFontDefinition> result = new();
      StartCoroutine(LoadResourceAsync<Font>(fontAddress.Address, f =>
      {
        if (f == null)
        {
          Debug.LogError($"Font not found: {fontAddress.Address}");
          result.SetResult(new StyleFontDefinition(StyleKeyword.Null));
        }
        else
        {
          var font = new StyleFontDefinition(f);
          _fonts[fontAddress.Address] = font;
          result.SetResult(font);
        }
      }));

      return result.Task;
    }

    IEnumerator<YieldInstruction> LoadResourceAsync<T>(string address, Action<T?> onComplete) where T : Object
    {
      var load = Resources.LoadAsync<T>(address);
      yield return load;
      if (load.asset is T result)
      {
        onComplete(result);
      }
      else
      {
        onComplete(null);
      }
    }
  }
}