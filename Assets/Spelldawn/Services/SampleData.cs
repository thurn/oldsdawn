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

using System.Collections.Generic;
using Spelldawn.Protos;
using UnityEngine;
using static Spelldawn.Masonry.MasonUtil;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class SampleData : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    int _lastReturnedCard;

    static readonly string Text1 = @"<sprite name=""hourglass"">, 2<sprite name=""fire"">: Destroy this token. Add another line of text to this card.
<b><sprite name=""bolt"">Play</b>: Give the Overlord the <u>Shatter</u> ability";

    static readonly string Text2 = @"If another item would be destroyed, this item is destroyed instead
";

    static readonly string Text3 = @"<b><sprite name=""bolt"">Play:</b> <b>Store</b> 12<sprite name=""fire"">
<sprite name=""hourglass"">: <b>Take</b> 2<sprite name=""fire"">
";

    static readonly string Text4 =
      @"Search your deck for a weapon. If you made a successful raid this turn you may play it <i>(paying its costs)</i>, otherwise put it into your hand
";

    static readonly string Text5 =
      @"Choose a minion. That minion gains the <b>Infernal</b>, <b>Human</b>, and <b>Abyssal</b> subtypes until end of turn.
";

    static readonly string Text6 = @"<b>Choose One:</b>
• Gain 2<sprite name=""fire"">
• Reveal one card
";

    static readonly string Text7 = @"1<sprite name=""fire""> <b>Recurring</b> <i>(Refill up to 1<sprite name=""fire""> each turn)</i>
Use this <sprite name=""fire""> to pay for using Weapons or equipping Silver items 
";

    static readonly string Text8 = @"<b>Attack:</b> 2<sprite name=""fire"">: 1 damage
<b>Strike</b> 2 <i>(deal 2 damage at the start of combat)</i>
<b><sprite name=""bolt"">Play:</b> Pick a minion in play. Whenever you pay that minion’s shield cost, kill it
";

    static readonly string Text9 = @"<b>Attack:</b> 1<sprite name=""fire"">: 1 damage
<sprite name=""hourglass"">: Place a <sprite name=""dot""> on this item
When you use this item, remove a <sprite name=""dot""> or sacrifice it
";

    static readonly string Text10 = @"<b>Attack:</b> 1<sprite name=""fire"">: 1 damage
<b>Strike</b> 2 <i>(deal 2 damage at the start of combat)</i>
<b>Area</b> <i>(this item’s damage persists for the duration of the raid)</i>
";

    static readonly List<CardView> Cards = new()
    {
      RevealedCard(1, "Meteor Shower", Text1, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_21"),
      RevealedCard(2, "The Maker's Eye", Text2, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_25",
        "Poneti/4000_Fantasy_Icons/Z_Other/fr_bg/bg_blue"),
      RevealedCard(3, "Gordian Blade", Text3, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_13"),
      RevealedCard(4, "Personal Touch", Text4, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_16",
        "Poneti/4000_Fantasy_Icons/Z_Other/fr_bg/bg_green"),
      RevealedCard(5, "Secret Key", Text5, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_41",
        "Poneti/4000_Fantasy_Icons/Z_Other/fr_bg/bg_grey"),
      RevealedCard(6, "Femme Fatale", Text6, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_37"),
      RevealedCard(7, "Magic Missile", Text7, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_40"),
      RevealedCard(9, "Sleep Ray", Text9, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_66"),
      RevealedCard(8, "Divine Bolt", Text8, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_52"),
      RevealedCard(10, "Hideous Laughter", Text10, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_68")
    };

    void Start()
    {
      HandleCommands(new GameCommand
      {
        RenderGame = new RenderGameCommand
        {
          Game = SampleGame()
        }
      });
    }

    public void FakeActionResponse(GameAction action)
    {
      switch (action.ActionCase)
      {
        case GameAction.ActionOneofCase.DrawCard:
          DrawCardResponse();
          break;
      }
    }

    void DrawCardResponse()
    {
      var card = Card();
      HandleCommands(new GameCommand
      {
        CreateCard = new CreateCardCommand
        {
          Card = card,
          Position = CreateCardPosition.UserDeck
        }
      }, new GameCommand
      {
        MoveCard = new MoveCardCommand
        {
          CardId = card.CardId,
          TargetPlayer = PlayerName.User,
          Zone = GameZone.Hand
        }
      });
    }

    void HandleCommands(params GameCommand[] commands)
    {
      var list = new CommandList();
      list.Commands.AddRange(commands);
      StartCoroutine(_registry.CommandService.HandleCommands(list));
    }

    CardView Card() => Cards[_lastReturnedCard++ % 10];

    static GameView SampleGame() =>
      new()
      {
        User = Player("User", PlayerName.User,
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Backs/Back_Steampunk_Style_Color_1"),
        Opponent = Player("Opponent", PlayerName.Opponent,
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Backs/Back_Elf_Style_Color_1"),
        Arena = new ArenaView()
      };

    static CardId CardId(int id) => new() { Value = id };

    static PlayerView Player(string playerName, PlayerName id, string cardBack) => new()
    {
      PlayerInfo = new PlayerInfo
      {
        Name = playerName,
        IdentityCard = new CardView
        {
          CardId = CardId((int)id),
          CardBack = new SpriteAddress
          {
            Address = cardBack
          }
        }
      },
      Score = new ScoreView
      {
        Score = 0
      },
      Hand = new HandView(),
      Mana = new ManaView
      {
        Amount = 5
      },
      DiscardPile = new DiscardPileView(),
      ActionTracker = new ActionTrackerView
      {
        AvailableActionCount = 3
      },
      Deck = new DeckView()
    };

    static CardView RevealedCard(int cardId, string title, string text, string image, string? imageBackground = null) =>
      new()
      {
        CardId = CardId(cardId),
        CardBack = Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Backs/Back_Steampunk_Style_Color_1"),
        RevealedCard = new RevealedCardView
        {
          CardFrame = Sprite(
            "LittleSweetDaemon/TCG_Card_Fantasy_Design/Cards/Card_Steampunk_Style_Color_1"),
          TitleBackground = Sprite(
            "LittleSweetDaemon/TCG_Card_Design/Magic_Card/Magic_Card_Face_Tape"),
          Jewel = Sprite(
            "LittleSweetDaemon/TCG_Card_Fantasy_Design/Jewels/Jewel_Steampunk_Color_01"),
          ImageBackground = imageBackground is null ? null : Sprite(imageBackground),
          Image = Sprite(image),
          Title = new CardTitle
          {
            Text = title
          },
          RulesText = new RulesText
          {
            Text = text
          },
        }
      };
  }
}