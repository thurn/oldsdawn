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
  [CustomEditor(typeof(ProjectileMover))]
  public sealed class ProjectileMoverEditor : Editor
  {
    public override void OnInspectorGUI()
    {
      DrawDefaultInspector();

      if (GUILayout.Button("Upgrade"))
      {
        var projectile = ((ProjectileMover)target);
        var added = projectile.gameObject.AddComponent<Projectile>();

        TimedEffect? flash = null;
        if (projectile.flash)
        {
          flash = projectile.flash.AddComponent<TimedEffect>();
        }

        TimedEffect? hit = null;
        if (projectile.hit)
        {
          hit = projectile.hit.AddComponent<TimedEffect>();
        }

        added.EditorSetEffects(flash, hit);

        DestroyImmediate(projectile.gameObject.GetComponent<Rigidbody>(), allowDestroyingAssets: true);
        DestroyImmediate(projectile.gameObject.GetComponent<SphereCollider>(), allowDestroyingAssets: true);
        DestroyImmediate(projectile, allowDestroyingAssets: true);
      }
    }
  }
}