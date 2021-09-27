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
using static Spelldawn.Masonry.MasonUtil;

#nullable enable

namespace Spelldawn.Battle
{
  public static class DeckNode
  {
    const float DeckAspectRatio = 0.72761194f;

    public static Node? Render(DeckView? deck) => deck is null
      ? null
      : new Node
      {
        Name = NodeNames.Deck,
        Style = new FlexStyle
        {
          Width = Dip(44),
          Height = Dip(44 / DeckAspectRatio),
          FlexShrink = 0,
          FixedBackgroundImageAspectRatio = true,
          BackgroundImage = Sprite("LittleSweetDaemon/TCG_Card_Design/Customized/ChampionDeck")
        },
        PressedStyle = new FlexStyle
        {
          BackgroundImageTintColor = MakeColor(Color.gray)
        },
        EventHandlers = new EventHandlers
        {
          ClickAction = new GameAction
          {
            DrawCard = new DrawCardAction()
          }
        }
      };
  }
}