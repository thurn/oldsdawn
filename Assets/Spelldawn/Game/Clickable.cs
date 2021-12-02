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
  public abstract class Clickable : MonoBehaviour
  {
    /// <summary>
    /// Invoked on mouse down. Should return true to consume the event or false if it was ignored.
    /// </summary>
    public virtual bool MouseDown()
    {
      return false;
    }

    /// <summary>
    /// Sent every frame while the mouse button is held down to objects which received <see cref="MouseDown"/>
    /// and returned true.
    /// </summary>
    public virtual void MouseDrag()
    {
    }

    /// <summary>
    /// Sent on *any* mouse up event, anywhere on screen, to objects which received a <see cref="MouseDown"/>
    /// event and returned true.
    /// </summary>
    public virtual void MouseUp()
    {
    }
  }
}