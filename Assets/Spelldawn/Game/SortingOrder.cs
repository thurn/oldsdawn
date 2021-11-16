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
using UnityEngine;
using UnityEngine.Rendering;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class SortingOrder
  {
    public static SortingOrder Create(GameContext gameContext, int index = 0) => new(gameContext, index);

    readonly GameContext _gameContext;
    readonly int _index;

    SortingOrder(GameContext gameContext, int index)
    {
      _gameContext = gameContext;
      _index = index;
    }

    public void ApplyTo(SortingGroup group)
    {
      group.sortingOrder = Position();
    }

    public void ApplyTo(Renderer renderer)
    {
      renderer.sortingOrder = Position();
    }

    int Position() => _index + _gameContext switch
    {
      GameContext.Arena => 100,
      GameContext.Deck => 200,
      GameContext.DiscardPile => 300,
      GameContext.ArenaRaidParticipant => 400,
      GameContext.RaidParticipant => 500,
      GameContext.Hand => 600,
      GameContext.Interface => 700,
      GameContext.Staging => 800,
      GameContext.Browser => 900,
      GameContext.Scored => 1000,
      GameContext.Effects => 1100,
      GameContext.Dragging => 1200,
      GameContext.UserMessage => 1300,
      GameContext.RewardBrowser => 1400,
      GameContext.InfoZoom => 1500,
      _ => throw new ArgumentOutOfRangeException()
    };
  }
}