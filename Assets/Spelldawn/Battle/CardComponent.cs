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

using Spelldawn.Masonry;
using static Spelldawn.Masonry.MasonUtil;
using Spelldawn.Protos;
using Spelldawn.Services;
using UnityEngine;
using UnityEngine.UIElements;
using WhiteSpace = Spelldawn.Protos.WhiteSpace;

#nullable enable

namespace Spelldawn.Battle
{
  public sealed class CardComponent : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] int _textNumber;
    CardView _cardView = null!;
    VisualElement? _element;

    static readonly string Text1 = @"If another item would be destroyed, this item is destroyed instead
";

    static readonly string Text2 = @"★, 2❋: Destroy this token. Add another line of text to this card.
<b>↯Play</b>: Give the Overlord the <u>Shatter</u> ability";

    static readonly string Text3 = @"<b>↯Play:</b> <b>Store</b> 12❋
★: <b>Take</b> 2❋
";

    static readonly string Text4 =
      @"Search your deck for a weapon. If you made a successful raid this turn you may play it <i>(paying its costs)</i>, otherwise put it into your hand
";

    static readonly string Text5 =
      @"Choose a minion. That minion gains the <b>Infernal</b>, <b>Human</b>, and <b>Abyssal</b> subtypes until end of turn.
";

    static readonly string Text6 = @"<b>Choose One:</b>
• Gain 2❋
• Reveal one card
";

    static readonly string Text7 = @"1❋ <b>Recurring</b> <i>(Refill up to 1❋ each turn)</i>
Use this ❋ to pay for using Weapons or equipping Silver items 
";

    static readonly string Text8 = @"<b>Attack:</b> 2❋ → 1 damage
<b>Strike</b> 2 <i>(deal 2 damage at the start of combat)</i>
<b>↯Play:</b> Pick a minion in play. Whenever you pay that minion’s shield cost, kill it
";

    static readonly string Text9 = @"<b>Attack:</b> 1❋ → 1 damage
★: Place a ◈ on this item
When you use this item, remove a ◈ or sacrifice it
";

    static readonly string Text10 = @"<b>Attack:</b> 1❋ → 1 damage
<b>Strike</b> 2 <i>(deal 2 damage at the start of combat)</i>
<b>Area</b> <i>(this item’s damage persists for the duration of the raid)</i>
";

    void Start()
    {
      _cardView = new CardView
      {
        CardBack = Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Backs/Back_Steampunk_Style_Color_1"),
        CardFrame = Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Cards/Card_Steampunk_Style_Color_1"),
        Webbing = Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Webbings/Webbing_Steampunk_Style_Color_1"),
        Jewel = Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Jewels/Jewel_Steampunk_Color_01"),
        Image = Sprite("Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_06"),
        Title = new CardTitle
        {
          Text = "Meteor Shower"
        },
        RulesText = new RulesText
        {
          Text = Text1
        },
        CanPlay = false
      };
    }

    void Update()
    {
      Render();
    }

    async void Render()
    {
      var sprite = await _registry.AssetService.LoadSprite(_cardView.CardBack);
      var rect = sprite.value.sprite.rect;
      var imageScale = MultiplerForTargetDip(100, rect.height);
      var cardWidth = Dip(100 * rect.width / rect.height);
      var rulesText = _textNumber switch
      {
        1 => Text1,
        2 => Text2,
        3 => Text3,
        4 => Text4,
        5 => Text5,
        6 => Text6,
        7 => Text7,
        8 => Text8,
        9 => Text9,
        10 => Text10,
        _ => Text1
      };

      _element = await Mason.Render(_registry, Column("Card",
        new FlexStyle
        {
          BackgroundImageScaleMultiplier = imageScale,
          Position = FlexPosition.Absolute,
          Width = cardWidth,
          Height = Dip(100),
          Scale = Scale(3f),
          Inset = LeftTopDip(150f, 125f)
        },
        Row(
          "CardImage",
          new FlexStyle
          {
            BackgroundImage = _cardView.Image,
            Position = FlexPosition.Absolute,
            Inset = LeftTopDip(4f, 6.5f),
            Width = Dip(58f),
            Height = Dip(58f)
          }),
        Row(
          "CardFrame",
          new FlexStyle
          {
            BackgroundImage = _cardView.CardFrame,
            BackgroundImageScaleMultiplier = imageScale,
            Position = FlexPosition.Absolute,
            Inset = GroupDip(0)
          }),
        Row(
          "Webbing",
          new FlexStyle
          {
            BackgroundImage = _cardView.Webbing,
            BackgroundImageScaleMultiplier = imageScale,
            Position = FlexPosition.Absolute,
            Inset = LeftTopDip(-2.15f, -4f)
          }),
        Row(
          "Jewel",
          new FlexStyle
          {
            BackgroundImage = _cardView.Jewel,
            BackgroundImageScaleMultiplier = imageScale,
            Position = FlexPosition.Absolute,
            Inset = LeftTopDip(31f, 65f)
          }),
        Text(_cardView.Title.Text,
          new FlexStyle
          {
            Position = FlexPosition.Absolute,
            Inset = LeftTopDip(0, -7.3f),
            Width = cardWidth,
            Height = Dip(0f),
            TextAlign = TextAlign.MiddleCenter,
            Color = MakeColor(Color.white),
            TextOutlineColor = MakeColor(Color.black),
            TextOutlineWidth = 0.1f,
            FontSize = Dip(5f),
            Font = Font("Fonts/Impact"),
            TextShadow = new Protos.TextShadow
            {
              Color = MakeColor(Color.black),
              Offset = new FlexVector2
              {
                X = 0.1f,
                Y = 0.1f
              },
              BlurRadius = 0.5f
            }
          }),
        Text($"<line-height=4>{rulesText}</line-height>",
          new FlexStyle
          {
            Position = FlexPosition.Absolute,
            Inset = LeftTopDip(4.9f, 67f),
            Width = Dip(53f),
            Height = Dip(25f),
            TextAlign = TextAlign.MiddleCenter,
            Color = MakeColor("#d7ccc8"),
            FontSize = Dip(4f),
            Font = Font("Fonts/Roboto"),
            WhiteSpace = WhiteSpace.Normal,
            ParagraphSpacing = Dip(25f)
          })
      ));

      _registry.Document.rootVisualElement.Clear();
      _registry.Document.rootVisualElement.Add(_element);
    }
  }
}