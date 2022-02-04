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
using Spelldawn.Utils;
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class GameService : MonoBehaviour
  {
    const int DefaultUserId = 1;
    const ulong DefaultGameId = 0;

    [SerializeField] Registry _registry = null!;

    public PlayerIdentifier PlayerId
    {
      get
      {
        if (PlayerPrefs.HasKey(Preferences.PlayerId) &&
            ulong.TryParse(PlayerPrefs.GetString(Preferences.PlayerId), out var playerId))
        {
          return new PlayerIdentifier
          {
            Value = playerId
          };
        }
        else
        {
          return new PlayerIdentifier
          {
            Value = DefaultUserId
          };
        }
      }
      set => PlayerPrefs.SetString(Preferences.PlayerId, value.Value.ToString());
    }

    public GameIdentifier? CurrentGameId
    {
      get
      {
        if (PlayerPrefs.HasKey(Preferences.CurrentGameId) &&
            ulong.TryParse(PlayerPrefs.GetString(Preferences.CurrentGameId), out var currentGameId))
        {
          return new GameIdentifier
          {
            Value = currentGameId
          };
        }
        else
        {
          return new GameIdentifier
          {
            Value = DefaultGameId
          };
        }
      }
      set => PlayerPrefs.SetString(Preferences.CurrentGameId, value?.Value.ToString());
    }

    void Start()
    {
      _registry.ActionService.Connect();
    }
  }
}