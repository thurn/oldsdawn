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
  public sealed class SceneBackground : MonoBehaviour
  {
    [SerializeField] bool _flipped;

    public void Flip()
    {
      if (_flipped)
      {
        transform.position = new Vector3(0, 0, 36);
        transform.localEulerAngles = new Vector3(0, 0, 0);
        _flipped = false;
      }
      else
      {
        transform.position = new Vector3(0, 0, 55);
        transform.localEulerAngles = new Vector3(0, 180, 0);
        _flipped = true;
      }
    }
  }
}