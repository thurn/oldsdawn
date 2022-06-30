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

#nullable enable

using System;
using Spelldawn.Game;
using UnityEngine;

namespace Spelldawn.Services
{
  public sealed class InputService : MonoBehaviour
  {
    readonly RaycastHit[] _raycastHitsTempBuffer = new RaycastHit[8];
    Clickable? _lastClicked;
    [SerializeField] Registry _registry = null!;

    void Update()
    {
      switch (Input.GetMouseButton(0))
      {
        case true when _lastClicked:
          _lastClicked!.MouseDrag();
          break;
        case true when !_lastClicked:
          _lastClicked = FireMouseDown();
          break;
        case false when _lastClicked:
          var last = _lastClicked;
          _lastClicked = null; // Do this first in case MouseUp() throws
          last!.MouseUp();
          break;
      }      
    }
    
    Clickable? FireMouseDown()
    {
      var ray = _registry.MainCamera.ScreenPointToRay(Input.mousePosition);
      var hits = Physics.RaycastNonAlloc(ray, _raycastHitsTempBuffer, 100);
      Clickable? fired = null;

      for (var i = 0; i < hits; ++i)
      {
        var hit = _raycastHitsTempBuffer[i];
        var clickable = hit.collider.GetComponent<Clickable>();
        if (clickable)
        {
          if (!fired)
          {
            var consumed = clickable.MouseDown();
            if (consumed)
            {
              fired = clickable;
            }
          }
        }
      }

      Array.Clear(_raycastHitsTempBuffer, 0, _raycastHitsTempBuffer.Length);
      return fired;
    }
    
  }
}