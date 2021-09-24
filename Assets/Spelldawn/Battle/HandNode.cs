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

using System.Linq;
using static Spelldawn.Masonry.MasonUtil;
using Spelldawn.Protos;

#nullable enable

namespace Spelldawn.Battle
{
  public static class HandNode
  {
    public static Node? Render(string playerName, HandView? handView) =>
      handView is null
        ? null
        : Row(
          $"{playerName}Hand",
          new FlexStyle
          {
            JustifyContent = FlexJustify.SpaceBetween,
            AlignItems = FlexAlign.Center,
            Margin = LeftRightDip(150),
            FlexGrow = 1
          },
          handView.Cards.Select((c, i) => CardNode.Render(c, (i + 1f) / (handView.Cards.Count + 1f))));
  }
}