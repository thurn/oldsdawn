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

using System;
using Spelldawn.Protos;
using Spelldawn.Services;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class TogglePanelButton : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] Panel _panel;

    enum Panel
    {
      GameMenu,
      Feedback
    }

    void OnMouseUpAsButton()
    {
      _registry.StaticAssets.PlayButtonSound();

      var address = new InterfacePanelAddress
      {
        ClientPanel = _panel switch
        {
          Panel.GameMenu => throw new NotImplementedException(),
          Panel.Feedback => ClientPanelAddress.DebugPanel,
          _ => throw new ArgumentOutOfRangeException()
        }
      };
      
      _registry.DocumentService.TogglePanel(!_registry.DocumentService.IsOpen(address), address);
    }
  }
}