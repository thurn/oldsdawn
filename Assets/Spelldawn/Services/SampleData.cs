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
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using Google.Protobuf.Collections;
using Google.Protobuf.WellKnownTypes;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEngine;
using static Spelldawn.Masonry.MasonUtil;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class SampleData : MonoBehaviour
  {
    const uint UserIdentityCardId = 1234;
    const uint OpponentIdentityCardId = 1235;
    [SerializeField] Registry _registry = null!;
    [SerializeField] StartBehavior _startBehavior;
    uint _lastReturnedCard;
    uint _lastOpponentCardId = 65536;
    readonly List<CardIdentifier> _opponentHandCards = new();
    readonly List<CardIdentifier> _opponentPlayedCards = new();

    public enum StartBehavior
    {
      Unspecified,
      DrawImmediately,
      ShowOpeningHand
    }

    static readonly string Text1 =
      @"<sprite name=""hourglass"">, 2<sprite name=""fire"">: Destroy this token. Add another line of text to this card.
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

    static readonly string Text7 =
      @"1<sprite name=""fire""> <b>Recurring</b> <i>(Refill up to 1<sprite name=""fire""> each turn)</i>
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

    CardView _card1 = null!;

    static List<CardView> _cards = null!;

    enum CardType
    {
      Abyssal,
      Infernal,
      Mortal,
      Artifact,
      Spell
    }

    IEnumerator Start()
    {
      if (_startBehavior == StartBehavior.Unspecified)
      {
        yield break;
      }

      _card1 = RevealedUserCard(1, "Meteor Shower", Text1,
        "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_21", CardType.Spell, 0);

      _cards = new List<CardView>
      {
        _card1,
        RevealedUserCard(2, "The Maker's Eye", Text2, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_25",
          CardType.Abyssal, 4),
        RevealedUserCard(3, "Gordian Blade", Text3, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_13",
          CardType.Infernal, 1),
        RevealedUserCard(4, "Personal Touch", Text4, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_16",
          CardType.Mortal, 15, showExtraHelpers: true),
        RevealedUserCard(5, "Secret Key", Text5, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_42",
          CardType.Spell, 4),
        RevealedUserCard(6, "Femme Fatale", Text6, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_37",
          CardType.Artifact, 2),
        RevealedUserCard(7, "Magic Missile", Text7, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_40",
          CardType.Abyssal, 3),
        RevealedUserCard(9, "Sleep Ray", Text9, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_66",
          CardType.Infernal, 0),
        RevealedUserCard(8, "Divine Bolt", Text8, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_52",
          CardType.Spell, 3),
        RevealedUserCard(10, "Hideous Laughter", Text10, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_68",
          CardType.Spell, 6)
      };

      yield return _registry.CommandService.HandleCommands(new GameCommand
      {
        UpdateGameView = new UpdateGameViewCommand
        {
          Game = SampleGame()
        }
      }, new GameCommand
      {
        CreateOrUpdateCard = new CreateOrUpdateCardCommand
        {
          Card = RevealedUserCard(UserIdentityCardId, "User Identity", "Identity Card Text",
            "Enixion/Fantasy Art Pack 2/Resized/2"),
          CreatePosition = new ObjectPosition
          {
            SortingKey = 1,
            Identity = new ObjectPositionIdentity
            {
              Owner = PlayerName.User
            }
          }
        }
      }, new GameCommand
      {
        CreateOrUpdateCard = new CreateOrUpdateCardCommand
        {
          Card = OpponentCard("Opponent Identity", OpponentIdentityCardId, 12),
          CreatePosition = new ObjectPosition
          {
            SortingKey = 1,
            Identity = new ObjectPositionIdentity
            {
              Owner = PlayerName.Opponent
            }
          }
        }
      });

      switch (_startBehavior)
      {
        case StartBehavior.DrawImmediately:
          yield return DrawImmediately();
          break;
        case StartBehavior.ShowOpeningHand:
          yield return new WaitForSeconds(0.5f);
          yield return ShowOpeningHand();
          break;
      }
    }

    public static GameObjectIdentifier CardObjectId(uint cardId) => IdUtil.CardObjectId(CardId(cardId));

    public static CardIdentifier CardId(uint cardId) => new()
    {
      Side = PlayerSide.Champion,
      Index = cardId
    };

    IEnumerator DrawImmediately()
    {
      if (_startBehavior == StartBehavior.DrawImmediately)
      {
        for (var i = 0; i < 6; ++i)
        {
          yield return DrawUserCard(directToHand: true);
          yield return DrawOpponentCard(disableAnimation: true);
        }
      }
    }

    IEnumerator ShowOpeningHand()
    {
      var cards = _cards.Take(5).ToList();
      return _registry.CommandService.HandleCommands(
        cards.Select(c => new GameCommand
        {
          CreateOrUpdateCard = new CreateOrUpdateCardCommand
          {
            Card = c,
            CreatePosition = DeckPosition(PlayerName.User, c.CardId.Index)
          }
        }).Interleave(cards.Select(c => new GameCommand
        {
          MoveGameObjects = new MoveGameObjectsCommand
          {
            Ids = { IdUtil.CardObjectId(c.CardId) },
            Position = new ObjectPosition
            {
              SortingKey = c.CardId.Index,
              Browser = new ObjectPositionBrowser()
            }
          }
        })).Append(new GameCommand
        {
          RenderInterface = new RenderInterfaceCommand
          {
            MainControls = new InterfaceMainControls
            {
              Node = MulliganControls(cards)
            }
          }
        }));
    }

    Node MulliganControls(IEnumerable<CardView> cards) => Row("ControlButtons",
      new FlexStyle
      {
        JustifyContent = FlexJustify.Center,
        FlexGrow = 1,
        AlignItems = FlexAlign.Center,
      },
      Button("Keep", action: KeepHandAction(cards), orange: true),
      Button("Mulligan"));

    StandardAction KeepHandAction(IEnumerable<CardView> cards) =>
      new()
      {
        DebugPayload = Any.Pack(new CommandList
        {
          Commands = { Delay(1000) }
        }),
        Update = new CommandList
        {
          Commands =
          {
            ClearMainControls(),
            new GameCommand
            {
              MoveGameObjects = new MoveGameObjectsCommand
              {
                Ids = { cards.Select(IdUtil.CardObjectId) },
                Position = new ObjectPosition
                {
                  SortingKey = 1,
                  Hand = new ObjectPositionHand
                  {
                    Owner = PlayerName.User
                  }
                }
              }
            }
          }
        }
      };

    void Update()
    {
      if (Input.GetKeyDown(KeyCode.D))
      {
        StartCoroutine(DrawOpponentCard());
      }

      if (Input.GetKeyDown(KeyCode.F))
      {
        StartCoroutine(PlayOpponentCard());
      }

      if (Input.GetKeyDown(KeyCode.G))
      {
        StartCoroutine(RevealOpponentCard());
      }

      if (Input.GetKeyDown(KeyCode.M))
      {
        StartCoroutine(_registry.CommandService.HandleCommands(new GameCommand
        {
          DisplayGameMessage = new DisplayGameMessageCommand
          {
            MessageType = GameMessageType.Dawn
          }
        }));
      }

      if (Input.GetKeyDown(KeyCode.N))
      {
        StartCoroutine(_registry.CommandService.HandleCommands(new GameCommand
        {
          DisplayGameMessage = new DisplayGameMessageCommand
          {
            MessageType = GameMessageType.Dusk
          }
        }));
      }

      if (Input.GetKeyDown(KeyCode.V))
      {
        StartCoroutine(_registry.CommandService.HandleCommands(
          new GameCommand
          {
            SetGameObjectsEnabled = new SetGameObjectsEnabledCommand
            {
              GameObjectsEnabled = false
            }
          },
          new GameCommand
          {
            DisplayGameMessage = new DisplayGameMessageCommand
            {
              MessageType = GameMessageType.Victory
            }
          },
          DisplayRewards()));
      }

      if (Input.GetKeyDown(KeyCode.B))
      {
        StartCoroutine(_registry.CommandService.HandleCommands(
          new GameCommand
          {
            SetGameObjectsEnabled = new SetGameObjectsEnabledCommand
            {
              GameObjectsEnabled = false
            }
          },
          new GameCommand
          {
            DisplayGameMessage = new DisplayGameMessageCommand
            {
              MessageType = GameMessageType.Defeat
            }
          }));
      }
    }

    public IEnumerator FakeActionResponse(GameAction action)
    {
      switch (action.ActionCase)
      {
        case GameAction.ActionOneofCase.StandardAction:
          return action.StandardAction.DebugPayload is { } p
            ? _registry.CommandService.HandleCommands(p.Unpack<CommandList>())
            : CollectionUtils.Yield();
        case GameAction.ActionOneofCase.DrawCard:
          return DrawUserCard();
        case GameAction.ActionOneofCase.LevelUpRoom:
          return LevelUpRoom();
        default:
          return CollectionUtils.Yield();
      }
    }

    IEnumerator DrawUserCard(bool directToHand = false)
    {
      var card = Card();
      var position = directToHand
        ? HandPosition(PlayerName.User, card.CardId.Index)
        : DeckPosition(PlayerName.User, card.CardId.Index);

      yield return _registry.CommandService.HandleCommands(new GameCommand
      {
        CreateOrUpdateCard = new CreateOrUpdateCardCommand
        {
          Card = card,
          CreatePosition = position,
          CreateAnimation = directToHand ? CardCreationAnimation.Unspecified : CardCreationAnimation.DrawCard,
          DisableFlipAnimation = directToHand
        }
      });

      if (!directToHand)
      {
        yield return _registry.CommandService.HandleCommands(new GameCommand
        {
          MoveGameObjects = new MoveGameObjectsCommand
          {
            Ids =
            {
              new GameObjectIdentifier
              {
                CardId = card.CardId
              }
            },
            Position = HandPosition(PlayerName.User, card.CardId.Index),
            DisableAnimation = directToHand
          }
        });
      }
    }

    static ObjectPosition DeckPosition(PlayerName playerName, uint sortingKey)
    {
      return new ObjectPosition
      {
        SortingKey = sortingKey,
        Deck = new ObjectPositionDeck
        {
          Owner = playerName
        }
      };
    }

    static ObjectPosition HandPosition(PlayerName playerName, uint sortingKey)
    {
      return new ObjectPosition
      {
        SortingKey = sortingKey,
        Hand = new ObjectPositionHand
        {
          Owner = playerName
        }
      };
    }

    IEnumerator DrawOpponentCard(bool disableAnimation = false)
    {
      var cardId = CardId(_lastOpponentCardId++);
      _opponentHandCards.Add(cardId);
      return _registry.CommandService.HandleCommands(new GameCommand
      {
        CreateOrUpdateCard = new CreateOrUpdateCardCommand
        {
          Card = new CardView
          {
            CardId = cardId,
            OwningPlayer = PlayerName.Opponent,
            ArenaFrame = Sprite("SpriteWay/Icons/Clean Frames/9048")
          },
          CreatePosition = DeckPosition(PlayerName.Opponent, cardId.Index)
        }
      }, new GameCommand
      {
        MoveGameObjects = new MoveGameObjectsCommand
        {
          Ids =
          {
            new GameObjectIdentifier
            {
              CardId = cardId
            }
          },
          Position = HandPosition(PlayerName.Opponent, cardId.Index),
          DisableAnimation = disableAnimation
        }
      });
    }

    IEnumerator PlayOpponentCard()
    {
      var cardId = _opponentHandCards[0];
      _opponentHandCards.RemoveAt(0);
      _opponentPlayedCards.Add(cardId);

      return _registry.CommandService.HandleCommands(MoveToRoom(cardId.Index, RoomIdentifier.RoomB));
    }

    IEnumerator RevealOpponentCard()
    {
      var cardId = _opponentPlayedCards[0];
      return _registry.CommandService.HandleCommands(
        MoveToStaging(cardId.Index),
        UpdateCard(OpponentCard("Scheme Card", cardId.Index, 19, revealedInArena: false)),
        Delay(1000),
        MoveToRoom(cardId.Index, RoomIdentifier.RoomB)
      );
    }

    CardView Card() => _cards[(int)_lastReturnedCard++ % 10];

    GameView SampleGame() =>
      new()
      {
        User = Player(
          "User",
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Backs/Back_Steampunk_Style_Color_1"),
        Opponent = Player(
          "Opponent",
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Backs/Back_Elf_Style_Color_1"),
        CurrentPriority = PlayerName.User
      };

    static TimeValue TimeMs(uint ms) => new() { Milliseconds = ms };

    static PlayerView Player(string playerName, string cardBack) => new()
    {
      PlayerInfo = new PlayerInfo
      {
        Name = playerName,
        CardBack = new SpriteAddress
        {
          Address = cardBack
        }
      },
      Score = new ScoreView
      {
        Score = 0
      },
      Mana = new ManaView
      {
        Amount = 5
      },
      ActionTracker = new ActionTrackerView
      {
        AvailableActionCount = 3
      },
    };

    static bool IsItem(uint cardId) => cardId % 2 == 0;

    CardView RevealedUserCard(
      uint cardId,
      string title,
      string text,
      string image,
      CardType cardType = CardType.Artifact,
      uint? manaCost = null,
      bool showExtraHelpers = false)
    {
      var roomTarget = new CardTargeting
      {
        PickRoom = new PickRoom()
      };

      var roomPos = new ObjectPosition
      {
        SortingKey = cardId,
        Room = new ObjectPositionRoom
        {
          RoomLocation = ClientRoomLocation.Front
        }
      };

      var itemPos = new ObjectPosition
      {
        SortingKey = cardId,
        Item = new ObjectPositionItem
        {
          ItemLocation = ClientItemLocation.Left
        }
      };

      return new CardView
      {
        CardId = CardId(cardId),
        OwningPlayer = PlayerName.User,
        ArenaFrame = Sprite(cardType switch
        {
          CardType.Abyssal => "SpriteWay/Icons/Clean Frames/9055",
          CardType.Infernal => "SpriteWay/Icons/Clean Frames/9054",
          CardType.Mortal => "SpriteWay/Icons/Clean Frames/9048",
          CardType.Artifact => "SpriteWay/Icons/Clean Frames/9047",
          CardType.Spell => "SpriteWay/Icons/Clean Frames/9020",
          _ => throw new ArgumentOutOfRangeException(nameof(cardType), cardType, null)
        }),

        CardIcons = new CardIcons
        {
          TopLeftIcon = manaCost == null
            ? null
            : new CardIcon
            {
              Background = Sprite("LittleSweetDaemon/TCG_Card_Fantasy_Design/Icons/Icon_Mana_Color_01"),
              Text = manaCost + ""
            },
          BottomLeftIcon = cardType != CardType.Infernal
            ? null
            : new CardIcon
            {
              Background = Sprite("LittleSweetDaemon/TCG_Card_Elemental_Design/Number_Icons/Number_Icons_Color_6"),
              Text = "1",
              BackgroundScale = 1.1f
            },
          BottomRightIcon = cardType switch
          {
            CardType.Infernal => new CardIcon
            {
              Background = Sprite("LittleSweetDaemon/TCG_Card_Elemental_Design/Heart_Icons/Heart_Icons_Color_5"),
              Text = "4",
              BackgroundScale = 1.5f
            },
            CardType.Mortal => new CardIcon
            {
              Background =
                Sprite("LittleSweetDaemon/TCG_Card_Elemental_Design/Attack_Icons/Attack_Icons_Color_4"),
              Text = "4",
              BackgroundScale = 1.75f
            },
            _ => null
          }
        },

        RevealedCard = new RevealedCardView
        {
          CardFrame = Sprite(
            "LittleSweetDaemon/TCG_Card_Fantasy_Design/Cards/Card_Steampunk_Style_Color_1"),
          TitleBackground = Sprite(cardType switch
          {
            CardType.Abyssal => "LittleSweetDaemon/TCG_Card_Design/Magic_Card/Magic_Card_Face_Tape",
            CardType.Infernal => "LittleSweetDaemon/TCG_Card_Design/Animal_Card/Animal_Card_Face_Tape",
            CardType.Mortal => "LittleSweetDaemon/TCG_Card_Design/Nautical_Card/Nautical_Card_Face_Tape",
            CardType.Artifact => "LittleSweetDaemon/TCG_Card_Design/Nautical_Card/Nautical_Card_Face_Tape",
            CardType.Spell => "LittleSweetDaemon/TCG_Card_Design/Warrior_Card/Warrior_Card Face_Tape",
            _ => throw new ArgumentOutOfRangeException(nameof(cardType), cardType, null)
          }),
          Jewel = Sprite(
            "LittleSweetDaemon/TCG_Card_Fantasy_Design/Jewels/Jewel_Steampunk_Color_01"),
          Image = Sprite(image),
          Title = new CardTitle
          {
            Text = title
          },
          RulesText = new RulesText
          {
            Text = text
          },
          Targeting = IsItem(cardId) ? null : roomTarget,
          OnReleasePosition = IsItem(cardId) ? itemPos : roomPos,
          CanPlay = true,
          RevealedInArena = true,
          SupplementalInfo = SupplementalInfo(cardType switch
          {
            CardType.Abyssal => "Weapon • Abyssal",
            CardType.Infernal => "Weapon • Infernal",
            CardType.Mortal => "Weapon • Mortal",
            CardType.Artifact => "Artifact",
            CardType.Spell => "Spell",
            _ => throw new ArgumentOutOfRangeException(nameof(cardType), cardType, null)
          }, showExtraHelpers)
        }
      };
    }

    CardView OpponentCard(string title, uint cardId, uint image, bool revealedInArena = false) => new()
    {
      CardId = CardId(cardId),
      CardIcons = new CardIcons
      {
        TopLeftIcon = new CardIcon
        {
          Background = Sprite("LittleSweetDaemon/TCG_Card_Fantasy_Design/Icons/Icon_Mana_Color_01"),
          Text = "4"
        }
      },
      RevealedCard = new RevealedCardView
      {
        CardFrame = Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Cards/Card_Elf_Style_Color_1"),
        TitleBackground = Sprite(
          "LittleSweetDaemon/TCG_Card_Design/Magic_Card/Magic_Card_Face_Tape"),
        Jewel = Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Jewels/Jewel_Elf_Color_01"),
        Image = Sprite($"Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_{image}"),
        Title = new CardTitle
        {
          Text = title
        },
        RulesText = new RulesText
        {
          Text = "Some card text"
        },
        RevealedInArena = revealedInArena
      }
    };

    Node CryptsRaidControls() => Row("ControlButtons",
      new FlexStyle
      {
        JustifyContent = FlexJustify.FlexEnd,
        FlexGrow = 1,
        AlignItems = FlexAlign.Center,
        Wrap = FlexWrap.WrapReverse,
      }, Button("Continue", action: AccessDiscardPileAction()));

    StandardAction AccessDiscardPileAction() => new()
    {
      Update = new CommandList
      {
        Commands =
        {
          FireProjectile(IdUtil.IdentityCardId(PlayerName.User), IdUtil.DiscardPileObjectId(PlayerName.Opponent), 4)
        }
      },
      DebugPayload = Any.Pack(new CommandList
      {
        Commands =
        {
          ClearMainControls(),
          RunInParallel(
            MoveIdentityToContainer(PlayerName.User),
            MoveDiscardPileToContainer(PlayerName.Opponent)
          )
        }
      })
    };

    Node SanctumRaidControls() => Row("ControlButtons",
      new FlexStyle
      {
        JustifyContent = FlexJustify.FlexEnd,
        FlexGrow = 1,
        AlignItems = FlexAlign.Center,
        Wrap = FlexWrap.WrapReverse,
      }, Button("Continue", action: AccessHandAction(RoomIdentifier.Sanctum)));

    StandardAction AccessHandAction(RoomIdentifier fromRoom) => new()
    {
      Update = new CommandList
      {
        Commands =
        {
          FireProjectile(CardObjectId(1), IdUtil.IdentityCardId(PlayerName.User), 3)
        }
      },
      DebugPayload = Any.Pack(new CommandList
      {
        Commands =
        {
          MoveToRoom(1, RoomIdentifier.Sanctum),
          FireProjectile(IdUtil.IdentityCardId(PlayerName.User), IdUtil.IdentityCardId(PlayerName.Opponent), 4),
          ClearMainControls(),
          RunInParallel(
            MoveIdentityToContainer(PlayerName.User),
            MoveIdentityToContainer(PlayerName.Opponent)
          ),
          UpdateCard(OpponentCard("Revealed Card", 65539, 18)),
          UpdateCard(OpponentCard("Scheme Card", 65541, 19)),
          RunInParallel(_opponentHandCards.Select(id => MoveToBrowser(IdUtil.CardObjectId(id)))),
          RenderCardButton(CardId(65541), "Score!", ScoreAction(65541,
            RunInParallel(_opponentHandCards
              .Except(CollectionUtils.Once(CardId(65541)))
              .Select(id => MoveToHand(id.Index, PlayerName.Opponent)))))
        }
      })
    };

    GameCommand UpdateCard(CardView cardView) => new()
    {
      CreateOrUpdateCard = new CreateOrUpdateCardCommand
      {
        Card = cardView
      }
    };

    Node VaultRaidControls() => Row("ControlButtons",
      new FlexStyle
      {
        JustifyContent = FlexJustify.FlexEnd,
        FlexGrow = 1,
        AlignItems = FlexAlign.Center,
        Wrap = FlexWrap.WrapReverse,
      },
      Button("The Maker's Eye\n5\uf06d", action: CardStrikeAction(RoomIdentifier.Vault), smallText: true, orange: true),
      Button("Gordian Blade\n3\uf06d", action: null, smallText: true, orange: true),
      Button("Continue"));

    Node Button(string label, StandardAction? action = null, bool smallText = false, bool orange = false) => Row(
      "Button",
      new FlexStyle
      {
        Margin = AllDip(8),
        Height = Dip(88),
        MinWidth = Dip(88),
        JustifyContent = FlexJustify.Center,
        AlignItems = FlexAlign.Center,
        FlexShrink = 0,
        BackgroundImage = Sprite(orange
          ? "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/Buttons/Rescaled/Button_Orange"
          : "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/Buttons/Rescaled/Button_Gray"),
        ImageSlice = ImageSlice(0, 16)
      },
      new EventHandlers
      {
        OnClick = new GameAction
        {
          StandardAction = action
        }
      },
      Text(label, new FlexStyle
      {
        Margin = LeftRightDip(16),
        Padding = AllDip(0),
        Color = MakeColor(Color.white),
        FontSize = Dip(smallText ? 26 : 32),
        Font = Font("Fonts/Roboto"),
        TextAlign = TextAlign.MiddleCenter
      }));

    StandardAction CardStrikeAction(RoomIdentifier fromRoom) =>
      new()
      {
        DebugPayload = Any.Pack(AccessDeck()),
        Update = new CommandList
        {
          Commands =
          {
            ClearMainControls(),
            FireProjectile(
              CardObjectId(2),
              CardObjectId(1),
              projectileNumber: 8,
              additionalHit: true,
              hideOnHit: true,
              jumpToRoomOnHit: fromRoom)
          }
        }
      };

    GameCommand ClearMainControls() =>
      new()
      {
        RenderInterface = new RenderInterfaceCommand
        {
          MainControls = new InterfaceMainControls()
        }
      };

    CommandList AccessDeck() => new()
    {
      Commands =
      {
        new GameCommand
        {
          FireProjectile = new FireProjectileCommand
          {
            SourceId = IdUtil.IdentityCardId(PlayerName.User),
            TargetId = IdUtil.DeckObjectId(PlayerName.Opponent),
            Projectile = new ProjectileAddress
            {
              Address = "Hovl Studio/AAA Projectiles Vol 1/Prefabs/Projectiles/Projectile 1"
            },
            TravelDuration = TimeMs(300),
            WaitDuration = TimeMs(300)
          }
        },
        new GameCommand
        {
          CreateOrUpdateCard = new CreateOrUpdateCardCommand
          {
            Card = OpponentCard("Scheme", 55555, 92),
            CreatePosition = new ObjectPosition
            {
              SortingKey = 55555,
              Deck = new ObjectPositionDeck
              {
                Owner = PlayerName.Opponent
              }
            }
          }
        },
        MoveToRaidIndex(55555, 1),
        new GameCommand
        {
          CreateOrUpdateCard = new CreateOrUpdateCardCommand
          {
            Card = OpponentCard("Not A Scheme", 55556, 98),
            CreatePosition = new ObjectPosition
            {
              SortingKey = 55556,
              Deck = new ObjectPositionDeck
              {
                Owner = PlayerName.Opponent
              }
            }
          }
        },
        MoveToRaidIndex(55556, 1),
        MoveIdentityToContainer(PlayerName.User),
        RenderCardButton(CardId(55555), "Score!",
          ScoreAction(55555, MoveToDeck(55556, PlayerName.Opponent)))
      }
    };

    GameCommand RenderCardButton(CardIdentifier id, string label, StandardAction onClick)
    {
      return new GameCommand
      {
        RenderInterface = new RenderInterfaceCommand
        {
          CardAnchorNodes =
          {
              new CardAnchorNode
              {
                CardId = id,
                Node = Button(label, action: onClick, smallText: false, orange: true),
                AnchorPosition = CardNodeAnchorPosition.Bottom
              }
          }
        }
      };
    }

    GameCommand MoveIdentityToContainer(PlayerName playerName) =>
      new()
      {
        MoveGameObjects = new MoveGameObjectsCommand
        {
          Ids =
          {
            IdUtil.IdentityCardId(playerName)
          },
          Position = new ObjectPosition
          {
            SortingKey = 1,
            IdentityContainer = new ObjectPositionIdentityContainer
            {
              Owner = playerName
            }
          }
        }
      };

    GameCommand MoveDiscardPileToContainer(PlayerName playerName) =>
      new()
      {
        MoveGameObjects = new MoveGameObjectsCommand
        {
          Ids = { IdUtil.DiscardPileObjectId(playerName) },
          Position = new ObjectPosition
          {
            SortingKey = 1,
            DiscardPileContainer = new ObjectPositionDiscardPileContainer
            {
              Owner = playerName
            }
          }
        }
      };

    GameCommand MoveToRaidIndex(uint cardId, uint index) => MoveToRaidIndex(CardObjectId(cardId), index);

    GameCommand MoveToRaidIndex(GameObjectIdentifier id, uint index) => new()
    {
      MoveGameObjects = new MoveGameObjectsCommand
      {
        Ids = { id },
        Position = new ObjectPosition
        {
          SortingKey = index,
          Raid = new ObjectPositionRaid()
        }
      }
    };

    GameCommand MoveToBrowser(GameObjectIdentifier id) => new()
    {
      MoveGameObjects = new MoveGameObjectsCommand
      {
        Ids = { id },
        Position = new ObjectPosition
        {
          SortingKey = 1,
          Browser = new ObjectPositionBrowser()
        }
      }
    };

    StandardAction ScoreAction(uint cardId, GameCommand cleanUp) => new()
    {
      DebugPayload = Any.Pack(new CommandList
      {
        Commands =
        {
          MoveToDeckContainer(PlayerName.Opponent),
          MoveToIdentity(cardId),
          SetUserScore(2),
        }
      }),
      Update = new CommandList
      {
        Commands =
        {
          new GameCommand
          {
            RenderInterface = new RenderInterfaceCommand()
          },
          SetMusic(MusicState.Silent),
          PlaySound("Cafofo/Fantasy Music Pack Vol 1/Events/Positive Event 01"),
          MoveToScored(cardId),
          cleanUp,
          PlayHitEffect(cardId, 4, 700, "Universal Sound FX/FIREWORKS/FIREWORKS_Rocket_Explode_Large_RR1_mono"),
          PlayHitEffect(cardId, 4, 300, "Universal Sound FX/FIREWORKS/FIREWORKS_Rocket_Explode_RR1_mono")
        }
      }
    };

    GameCommand RunInParallel(params GameCommand[] commands) => RunInParallel(commands.ToList());

    GameCommand RunInParallel(IEnumerable<GameCommand> commands)
    {
      var result = new RunInParallelCommand();
      foreach (var command in commands)
      {
        result.Commands.Add(new CommandList
        {
          Commands = { command }
        });
      }

      return new GameCommand
      {
        RunInParallel = result
      };
    }

    GameCommand PlaySound(string address) => new()
    {
      PlaySound = new PlaySoundCommand
      {
        Sound = new AudioClipAddress
        {
          Address = address
        }
      }
    };

    GameCommand SetMusic(MusicState musicState) => new()
    {
      SetMusic = new SetMusicCommand
      {
        MusicState = musicState
      }
    };

    GameCommand PlayHitEffect(uint cardId, uint i, uint duration = 300, string? sound = null) =>
      new()
      {
        PlayEffect = new PlayEffectCommand
        {
          Effect = new EffectAddress
          {
            Address = $"Hovl Studio/Magic hits/Prefabs/Hit {i}"
          },
          Position = new PlayEffectPosition
          {
            GameObject = CardObjectId(cardId)
          },
          Duration = TimeMs(duration),
          Scale = 2.0f,
          Sound = sound is null
            ? null
            : new AudioClipAddress
            {
              Address = sound
            }
        }
      };

    // ReSharper disable once UnusedMember.Local
    GameCommand Delay(uint ms) => new()
    {
      Delay = new DelayCommand
      {
        Duration = TimeMs(ms)
      }
    };

    GameCommand MoveToScored(uint cardId) => new()
    {
      MoveGameObjects = new MoveGameObjectsCommand
      {
        Ids = { CardObjectId(cardId) },
        Position = new ObjectPosition
        {
          SortingKey = cardId,
          ScoreAnimation = new ObjectPositionScoreAnimation()
        }
      }
    };

    GameCommand MoveToIdentity(uint cardId) => new()
    {
      MoveGameObjects = new MoveGameObjectsCommand
      {
        Ids = { CardObjectId(cardId) },
        Position = new ObjectPosition
        {
          SortingKey = cardId,
          Identity = new ObjectPositionIdentity
          {
            Owner = PlayerName.User
          }
        }
      }
    };

    // ReSharper disable once UnusedMember.Local
    GameCommand MoveToOffscreen(uint cardId) => new()
    {
      MoveGameObjects = new MoveGameObjectsCommand
      {
        Ids = { CardObjectId(cardId) },
        Position = new ObjectPosition
        {
          SortingKey = cardId,
          Offscreen = new ObjectPositionOffscreen()
        }
      }
    };

    GameCommand SetUserScore(uint score) => new()
    {
      UpdateGameView = new UpdateGameViewCommand
      {
        Game = new GameView
        {
          User = new PlayerView
          {
            Score = new ScoreView
            {
              Score = score
            }
          }
        }
      }
    };

    GameCommand MoveToDeck(uint cardId, PlayerName owner) => new()
    {
      MoveGameObjects = new MoveGameObjectsCommand
      {
        Ids = { CardObjectId(cardId) },
        Position = new ObjectPosition
        {
          SortingKey = cardId,
          Deck = new ObjectPositionDeck
          {
            Owner = owner
          }
        }
      }
    };

    GameCommand MoveToHand(uint cardId, PlayerName owner) => new()
    {
      MoveGameObjects = new MoveGameObjectsCommand
      {
        Ids = { CardObjectId(cardId) },
        Position = new ObjectPosition
        {
          SortingKey = cardId,
          Hand = new ObjectPositionHand
          {
            Owner = owner
          }
        }
      }
    };

    GameCommand MoveToRoom(uint cardId, RoomIdentifier roomId) => new()
    {
      MoveGameObjects = new MoveGameObjectsCommand
      {
        Ids = { CardObjectId(cardId) },
        Position = new ObjectPosition
        {
          SortingKey = cardId,
          Room = new ObjectPositionRoom
          {
            RoomId = roomId,
            RoomLocation = ClientRoomLocation.Front
          }
        }
      }
    };

    GameCommand MoveToStaging(uint cardId) => new()
    {
      MoveGameObjects = new MoveGameObjectsCommand
      {
        Ids = { CardObjectId(cardId) },
        Position = new ObjectPosition
        {
          SortingKey = cardId,
          Staging = new ObjectPositionStaging()
        }
      }
    };

    GameCommand MoveToDeckContainer(PlayerName owner) => new()
    {
      MoveGameObjects = new MoveGameObjectsCommand
      {
        Ids =
        {
          new GameObjectIdentifier
          {
            Deck = owner
          }
        },
        Position = new ObjectPosition
        {
          SortingKey = 1,
          DeckContainer = new ObjectPositionDeckContainer
          {
            Owner = owner
          }
        }
      }
    };

    // ReSharper disable once UnusedMember.Local
    GameCommand DebugLog(string message) => new()
    {
      DebugLog = new DebugLogCommand
      {
        Message = message
      }
    };

    GameCommand FireProjectile(
      GameObjectIdentifier sourceId,
      GameObjectIdentifier targetId,
      uint projectileNumber,
      bool additionalHit = false,
      bool hideOnHit = false,
      RoomIdentifier? jumpToRoomOnHit = null) => new()
    {
      FireProjectile = new FireProjectileCommand
      {
        SourceId = sourceId,
        TargetId = targetId,
        Projectile = new ProjectileAddress
        {
          Address = $"Hovl Studio/AAA Projectiles Vol 1/Prefabs/Projectiles/Projectile {projectileNumber}"
        },
        TravelDuration = TimeMs(300),
        AdditionalHit = additionalHit
          ? new EffectAddress
          {
            Address = "Hovl Studio/Sword slash VFX/Prefabs/Sword Slash 1"
          }
          : null,
        AdditionalHitDelay = additionalHit ? TimeMs(100) : null,
        WaitDuration = TimeMs(300),
        HideOnHit = hideOnHit,
        JumpToPosition = jumpToRoomOnHit is { } r
          ? new ObjectPosition
          {
            SortingKey = targetId.CardId?.Index ?? 1,
            Room = new ObjectPositionRoom
            {
              RoomId = r,
              RoomLocation = ClientRoomLocation.Front
            }
          }
          : null
      }
    };

    IEnumerator LevelUpRoom()
    {
      var updated = _card1.Clone();
      updated.CardIcons.ArenaIcon = new CardIcon
      {
        Text = "1",
        Background = Sprite("LittleSweetDaemon/TCG_Card_Elemental_Design/Number_Icons/Number_Icons_Color_3")
      };
      return _registry.CommandService.HandleCommands(new GameCommand
      {
        CreateOrUpdateCard = new CreateOrUpdateCardCommand
        {
          Card = updated
        }
      }, new GameCommand
      {
        MoveGameObjects = new MoveGameObjectsCommand
        {
          Ids = { IdUtil.IdentityCardId(PlayerName.User) },
          Position = new ObjectPosition
          {
            SortingKey = 1,
            IdentityContainer = new ObjectPositionIdentityContainer
            {
              Owner = PlayerName.User
            }
          }
        }
      });
    }

    GameCommand DisplayRewards() =>
      new()
      {
        DisplayRewards = new DisplayRewardsCommand
        {
          Rewards =
          {
            RevealedUserCard(21, "Reward#1", "Card Text", "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_71",
              CardType.Abyssal,
              6),
            RevealedUserCard(22, "Reward#2", "Card Text", "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_72",
              CardType.Infernal,
              4),
            RevealedUserCard(23, "Reward#3", "Card Text", "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_73",
              CardType.Infernal,
              3),
            RevealedUserCard(24, "Reward#4", "Card Text", "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_74",
              CardType.Spell),
            RevealedUserCard(25, "Reward#5", "Card Text", "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_75",
              CardType.Spell,
              0)
          }
        }
      };

    Node SupplementalInfo(string cardType, bool showExtraHelpers) => Column(
      "SupplementalInfo",
      new FlexStyle
      {
        JustifyContent = FlexJustify.FlexStart,
        AlignItems = FlexAlign.FlexStart,
        Margin = LeftRightDip(16),
        MaxWidth = Dip(600),
        MaxHeight = Dip(600)
      },
      SupplementalInfoText(cardType, isFirst: true),
      showExtraHelpers ? SupplementalInfoText("<b>Store:</b> Place \uf06d on this card") : null,
      showExtraHelpers
        ? SupplementalInfoText("<b>Take:</b> Remove \uf06d from this card and add to your mana pool")
        : null,
      showExtraHelpers ? SupplementalInfoText("<u>Shatter:</u> 2\uf06d: Destroy target artifact") : null);

    Node SupplementalInfoText(string text, bool isFirst = false) => Row(
      $"SupplementalInfoText",
      new FlexStyle
      {
        Margin = isFirst ? BottomDip(4) : TopBottomDip(4),
        BackgroundColor = MakeColor(Color.black, 0.75f),
        BorderRadius = AllBordersRadiusDip(12),
        JustifyContent = FlexJustify.Center,
        AlignItems = FlexAlign.Center,
      },
      Text(text, new FlexStyle
      {
        Margin = AllDip(16),
        Padding = AllDip(0),
        Color = MakeColor(Color.white),
        FontSize = Dip(32),
        Font = Font("Fonts/Roboto"),
        TextAlign = TextAlign.MiddleLeft,
        WhiteSpace = WhiteSpace.Normal
      }));
  }
}