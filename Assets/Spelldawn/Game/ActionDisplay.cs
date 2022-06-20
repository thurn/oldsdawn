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
using DG.Tweening;
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
    [SerializeField] Material _activeMaterial = null!;
    [SerializeField] Material _inactiveMaterial = null!;
    [SerializeField] TextMeshPro _number = null!;
    [SerializeField] TextMeshPro _left = null!;
    [SerializeField] TextMeshPro _center = null!;
    [SerializeField] TextMeshPro _right = null!;
    readonly Dictionary<TextMeshPro, bool> _filled = new();

    public uint AvailableActions => _availableActions;

    void Start()
    {
      _filled[_left] = true;
      _filled[_center] = true;
      _filled[_right] = true;
    }

    public void DisableAnimation()
    {
      var disabled = new Material(Shader.Find("TextMeshPro/Distance Field"));
      _left.fontMaterial = disabled;
      _center.fontMaterial = disabled;
      _right.fontMaterial = disabled;
      _activeMaterial = disabled;
      _inactiveMaterial = disabled;
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
          SetFilled(_left, false);
          SetFilled(_center, false);
          SetFilled(_right, false);
          break;
        case 1:
          SetFilled(_left, false);
          SetFilled(_center, false);
          SetFilled(_right, true);
          break;
        case 2:
          SetFilled(_left, false);
          SetFilled(_center, true);
          SetFilled(_right, true);
          break;
        case 3:
          SetFilled(_left, true);
          SetFilled(_center, true);
          SetFilled(_right, true);
          break;
        default:
          _left.gameObject.SetActive(false);
          _center.gameObject.SetActive(false);
          SetFilled(_right, true);
          _number.gameObject.SetActive(true);
          _number.text = availableActions + "";
          break;
      }
    }

    void SetFilled(TextMeshPro text, bool filled)
    {
      text.gameObject.SetActive(true);
      if (_filled[text] != filled)
      {
        _filled[text] = filled;
        TweenUtils
          .Sequence("ActionRotate")
          .Insert(0, text.transform.DOLocalRotate(filled ? new Vector3(0, 0, 180) : Vector3.zero, 0.3f))
          .InsertCallback(0.2f, () =>
          {
            text.fontMaterial = filled ? _activeMaterial : _inactiveMaterial;
            text.ForceMeshUpdate();
          });
      }
    }
  }
}