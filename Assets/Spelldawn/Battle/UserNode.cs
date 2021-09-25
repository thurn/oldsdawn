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
  public static class UserNode
  {
    public static Node? Render(PlayerView? playerView) =>
      playerView is null
        ? null
        : Row(
          "User",
          new FlexStyle(),
          Row("UserLeft", new FlexStyle
            {
              Width = Dip(150),
              JustifyContent = FlexJustify.SpaceAround,
              AlignContent = FlexAlign.FlexEnd,
              Margin = GroupDip(8, 25, 8, 8)
            },
            DiscardPileNode.Render(playerView.DiscardPile),
            DeckNode.Render(playerView.Deck)),
          HandNode.Render("User", playerView.Hand),
          Row("UserRight", new FlexStyle
          {
            Width = Dip(150),
            Margin = GroupDip(8, 8, 8, 25)
          })
        );
  }
}