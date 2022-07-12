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

using Spelldawn.Protos;
using Spelldawn.Utils;
using TMPro;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class ActionDisplay : MonoBehaviour
  {
    [SerializeField] uint _availableActions = 3;
    [SerializeField] TextMeshPro _number = null!;
    [SerializeField] ActionSymbol _left = null!;
    [SerializeField] ActionSymbol _center = null!;
    [SerializeField] ActionSymbol _right = null!;

    public uint AvailableActions => _availableActions;

    public bool IsAnimating => _left.IsAnimating || _center.IsAnimating || _right.IsAnimating;

    public void DisableAnimation()
    {
      var disabled = new Material(Shader.Find("TextMeshPro/Distance Field"));
      _left.SetFontMaterial(disabled);
      _center.SetFontMaterial(disabled);
      _right.SetFontMaterial(disabled);
    }    
    
    public void RenderActionTrackerView(ActionTrackerView actionTrackerView)
    {
      SetAvailableActions(actionTrackerView.AvailableActionCount);
    }

    public void SpendActions(uint amount)
    {
      Errors.CheckArgument(amount <= _availableActions, "Not enough actions available");
      SetAvailableActions(_availableActions - amount);
    }

    public void GainActions(uint amount)
    {
      SetAvailableActions(_availableActions + amount);
    }

    public void SetAvailableActions(uint availableActions)
    {
      _availableActions = availableActions;
      _number.gameObject.SetActive(false);

      switch (availableActions)
      {
        case 0:
          _left.SetFilled(false);
          _center.SetFilled(false);
          _right.SetFilled(false);
          break;
        case 1:
          _left.SetFilled(false);
          _center.SetFilled(false);
          _right.SetFilled(true);
          break;
        case 2:
          _left.SetFilled(false);
          _center.SetFilled(true);
          _right.SetFilled(true);
          break;
        case 3:
          _left.SetFilled(true);
          _center.SetFilled(true);
          _right.SetFilled(true);
          break;
        default:
          _left.gameObject.SetActive(false);
          _center.gameObject.SetActive(false);
          _right.SetFilled(true);
          _number.gameObject.SetActive(true);
          _number.text = availableActions + "";
          break;
      }
    }
  }
}