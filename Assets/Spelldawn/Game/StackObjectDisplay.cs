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

using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public abstract class StackObjectDisplay : ObjectDisplay
  {
    [SerializeField] float _singleElementY = 0.5f;

    protected override Vector3 CalculateObjectPosition(int index, int count) =>
      new(
        transform.position.x,
        transform.position.y + Mathf.Lerp(0f, 1f, count < 2 ? _singleElementY : index / ((float)count - 1)),
        transform.position.z);

    protected override Vector3? CalculateObjectRotation(int index, int count) => transform.rotation.eulerAngles;
  }
}