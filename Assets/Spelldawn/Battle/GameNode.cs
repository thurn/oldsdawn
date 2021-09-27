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
using static Spelldawn.Masonry.MasonUtil;

#nullable enable

namespace Spelldawn.Battle
{
  public static class GameNode
  {
    public static Node? Render(GameView? gameView) => gameView is null
      ? null
      : Column("Game",
        new FlexStyle
        {
          Position = FlexPosition.Absolute,
          Inset = AllDip(0),
          JustifyContent = FlexJustify.SpaceBetween
        },
        WithStyle(OpponentNode.Render(gameView.Opponent), style =>
        {
          style.Width = Percent(100);
          style.Height = Dip(50);
        }),
        WithStyle(ArenaNode.Render(gameView.Arena), style =>
        {
          style.Width = Percent(100);
          style.FlexGrow = 1;
        }),
        WithStyle(UserNode.Render(gameView.User), style =>
        {
          style.Width = Percent(100);
          style.Height = Dip(80);
        }),
        CardStagingNode.Render());
  }
}