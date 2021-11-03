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

using Spelldawn.Game;
using UnityEditor;
using UnityEngine;

#nullable enable

namespace Spelldawn.Editors
{
  [CustomEditor(typeof(TimedEffect))]
  public sealed class TimedEffectEditor : Editor
  {
    Color _color = Color.clear;

    public override void OnInspectorGUI()
    {
      DrawDefaultInspector();

      if (GUILayout.Button("Disabling Looping"))
      {
        foreach (var particleSystem in ((TimedEffect)target).GetComponentsInChildren<ParticleSystem>())
        {
          var main = particleSystem.main;
          main.loop = false;
        }
      }

      if (GUILayout.Button("Hierarchy Scaling"))
      {
        foreach (var particleSystem in ((TimedEffect)target).GetComponentsInChildren<ParticleSystem>())
        {
          var main = particleSystem.main;
          main.scalingMode = ParticleSystemScalingMode.Hierarchy;
        }
      }

      var color = EditorGUILayout.ColorField("Main Color", _color);
      if (color != _color && color != Color.clear)
      {
        _color = color;
      }

      if (GUILayout.Button("Set Start Color") && _color != Color.clear)
      {
        foreach (var particleSystem in ((TimedEffect)target).GetComponentsInChildren<ParticleSystem>())
        {
          var main = particleSystem.main;
          main.startColor = _color;
        }

        _color = Color.clear;
      }
    }
  }
}