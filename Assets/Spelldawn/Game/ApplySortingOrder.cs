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
using UnityEngine.Rendering;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class ApplySortingOrder : MonoBehaviour
  {
    [SerializeField] GameContext _gameContext;

    void OnValidate()
    {
      Apply();
    }

    void Start()
    {
      Apply();
    }

    void Apply()
    {
      var group = gameObject.GetComponent<SortingGroup>();
      if (!group)
      {
        group = gameObject.AddComponent<SortingGroup>();
      }

      SortingOrder.Create(_gameContext).ApplyTo(group);
    }
  }
}