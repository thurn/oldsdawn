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

namespace Spelldawn.Game
{
  public enum GameContext
  {
    // Note: Enum numbers are serialized by Unity and cannot be changed
    Unspecified = 0,
    Hidden = 17,
    Arena = 1,
    Deck = 2,
    DiscardPile = 3,
    Identity = 16,
    ArenaRaidParticipant = 10,
    RaidParticipant = 4,
    Hand = 5,
    Interface = 6,
    Browser = 12,
    Staging = 7,
    RevealedCardsBrowser = 18,
    Scored = 11,
    Effects = 8,
    Dragging = 9,
    UserMessage = 13,
    RewardBrowser = 14,
    InfoZoom = 15
  }

  public static class GameContextUtil
  {
    public static bool IsArenaContext(this GameContext gameContext) => gameContext switch
    {
      GameContext.Arena => true,
      GameContext.ArenaRaidParticipant => true,
      _ => false
    };
  }
}