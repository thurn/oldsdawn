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

using System.Collections;
using DG.Tweening;
using TMPro;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class ManaDisplay : MonoBehaviour
  {
    [SerializeField] TextMeshPro _manaText = null!;
    [SerializeField] TextMeshPro _manaSymbol = null!;

    void Start()
    {
      StartCoroutine(Flicker());
    }

    IEnumerator Flicker()
    {
      var glowValues = new[]
        { 0.19f, 0.2f, 0.25f, 0.35f, 0.2f, 0.4f, 0.1f, 0.25f, 0.2f, 0.4f, 0.35f, 0.4f, 0.2f, 0.4f };
      yield return new WaitForSeconds(Random.Range(1.0f, 5.0f));

      while (true)
      {
        foreach (var glow in glowValues)
        {
          yield return DOTween.To(
            () => _manaSymbol.fontMaterial.GetFloat(ShaderUtilities.ID_GlowOuter),
            val =>
            {
              _manaSymbol.fontMaterial.SetFloat(ShaderUtilities.ID_GlowOuter, val);
              _manaSymbol.UpdateMeshPadding();
            },
            endValue: 0.5f + glow,
            duration: 0.5f).WaitForCompletion();
        }
      }
    }
  }
}