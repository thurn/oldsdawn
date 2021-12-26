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

using Spelldawn.Protos;
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class GameService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;

    // ReSharper disable once NotAccessedField.Local
    [SerializeField] string? _debugGameId;

    GameIdentifier? _currentGameId;

    public GameIdentifier? CurrentGameId
    {
      get => _currentGameId;
      set
      {
        _debugGameId = value == null ? "None" : value.ToString();
        _currentGameId = value;
      }
    }

    void Start()
    {
      _registry.ActionService.HandleAction(new GameAction
      {
        Connect = new ConnectAction()
      });
    }
  }
}