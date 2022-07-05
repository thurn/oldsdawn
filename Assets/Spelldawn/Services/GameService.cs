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

using System.Collections;
using Spelldawn.Protos;
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class GameService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;

    PlayerIdentifier? _playerIdentifier;

    public void Initialize(GlobalGameMode globalGameMode)
    {
      if (globalGameMode == GlobalGameMode.Default)
      {
        StartCoroutine(Authenticate());
      }
    }

    IEnumerator Authenticate()
    {
      yield return new WaitForSeconds(0.1f);
      var identifier = new PlayerIdentifier
      {
        DeviceIdentifier = (Application.isEditor ? "Editor/" : "") + SystemInfo.deviceUniqueIdentifier
      };
      _registry.ActionService.Connect(identifier, offlineMode: Application.isMobilePlatform);
    }
  }
}