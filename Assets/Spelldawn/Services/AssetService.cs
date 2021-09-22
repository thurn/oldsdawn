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
using System.Threading.Tasks;
using Cysharp.Threading.Tasks;
using Spelldawn.Protos;
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class AssetService : MonoBehaviour
  {
    public static SpriteAddress Sprite(string address) => new SpriteAddress
    {
      Address = address
    };

    public async Task<Sprite> LoadSprite(SpriteAddress spriteAddress)
    {
      var result = await Resources.LoadAsync<Sprite>(spriteAddress.Address);
      if (result is Sprite s)
      {
        return s;
      }
      else
      {
        throw new InvalidOperationException($"Sprite not found: {spriteAddress.Address}");
      }
    }
  }
}