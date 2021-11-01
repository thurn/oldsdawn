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

using System.Collections;
using System.Collections.Generic;
using System.Linq;
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
    [SerializeField] Registry _registry = null!;
    [SerializeField] bool _drawHands;
    int _lastReturnedCard;
    int _lastOpponentCardId = 65536;
    readonly List<CardId> _opponentHandCards = new();

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

    static readonly CardView Card1 = RevealedCard(1, "Meteor Shower", Text1,
      "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_21", 0);

    static readonly List<CardView> Cards = new()
    {
      Card1,
      RevealedCard(2, "The Maker's Eye", Text2, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_25", 4,
        new CardTargeting
        {
          PickRoom = new PickRoom
          {
            ValidRooms =
            {
              RoomId.Crypts,
              RoomId.Sanctum,
              RoomId.Treasury
            }
          }
        }, new ObjectPosition
        {
          Room = new ObjectPositionRoom
          {
            RoomLocation = RoomLocation.Front
          }
        }),
      RevealedCard(3, "Gordian Blade", Text3, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_13", 1,
        releasePosition: new ObjectPosition
        {
          Item = new ObjectPositionItem
          {
            ItemLocation = ItemLocation.Left
          }
        }),
      RevealedCard(4, "Personal Touch", Text4, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_16", 15),
      RevealedCard(5, "Secret Key", Text5, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_42", 4),
      RevealedCard(6, "Femme Fatale", Text6, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_37", 2),
      RevealedCard(7, "Magic Missile", Text7, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_40", 3),
      RevealedCard(9, "Sleep Ray", Text9, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_66", 0),
      RevealedCard(8, "Divine Bolt", Text8, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_52", 3),
      RevealedCard(10, "Hideous Laughter", Text10, "Rexard/SpellBookPage01/SpellBookPage01_png/SpellBook01_68", 6)
    };

    IEnumerator Start()
    {
      yield return _registry.CommandService.HandleCommands(new GameCommand
      {
        RenderGame = new RenderGameCommand
        {
          Game = SampleGame()
        }
      });

      if (_drawHands)
      {
        for (var i = 0; i < 6; ++i)
        {
          yield return DrawUserCard(directToHand: true);
          yield return DrawOpponentCard(disableAnimation: true);
        }
      }
    }

    void Update()
    {
      if (Input.GetKeyDown(KeyCode.D))
      {
        StartCoroutine(DrawOpponentCard());
      }

      if (Input.GetKeyDown(KeyCode.R))
      {
        StartCoroutine(_registry.CommandService.HandleCommands(MoveToRaidIndex(55555, 1)));
      }
    }

    public IEnumerator FakeActionResponse(GameAction action)
    {
      switch (action.ActionCase)
      {
        case GameAction.ActionOneofCase.StandardAction:
          var commands = action.StandardAction.Payload.Unpack<CommandList>();
          return _registry.CommandService.HandleCommands(commands);
        case GameAction.ActionOneofCase.DrawCard:
          return DrawUserCard();
        case GameAction.ActionOneofCase.InitiateRaid:
          return InitiateRaid(action.InitiateRaid);
        case GameAction.ActionOneofCase.LevelUpRoom:
          return LevelUpRoom();
        default:
          return CollectionUtils.Yield();
      }
    }

    IEnumerator DrawUserCard(bool directToHand = false)
    {
      var card = Card();
      card.OnCreatePosition = directToHand ? HandPosition(PlayerName.User) : DeckPosition(PlayerName.User);

      yield return _registry.CommandService.HandleCommands(new GameCommand
      {
        CreateCard = new CreateCardCommand
        {
          Card = card,
          Animation = directToHand ? CardCreationAnimation.Unspecified : CardCreationAnimation.UserDeckToStaging,
          DisableAnimation = directToHand
        }
      });

      if (!directToHand)
      {
        yield return _registry.CommandService.HandleCommands(new GameCommand
        {
          MoveGameObject = new MoveGameObjectCommand
          {
            Id = new GameObjectId
            {
              CardId = card.CardId
            },
            Position = HandPosition(PlayerName.User),
            DisableAnimation = directToHand
          }
        });
      }
    }

    static ObjectPosition DeckPosition(PlayerName playerName)
    {
      return new ObjectPosition
      {
        Deck = new ObjectPositionDeck
        {
          Owner = playerName
        }
      };
    }

    static ObjectPosition HandPosition(PlayerName playerName)
    {
      return new ObjectPosition
      {
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
        CreateCard = new CreateCardCommand
        {
          Card = new CardView
          {
            CardId = cardId,
            OnCreatePosition = DeckPosition(PlayerName.Opponent),
            CardBack = Sprite("LittleSweetDaemon/TCG_Card_Fantasy_Design/Backs/Back_Elf_Style_Color_1")
          }
        }
      }, new GameCommand
      {
        MoveGameObject = new MoveGameObjectCommand
        {
          Id = new GameObjectId
          {
            CardId = cardId
          },
          Position = HandPosition(PlayerName.Opponent),
          DisableAnimation = disableAnimation
        }
      });
    }

    CardView Card() => Cards[_lastReturnedCard++ % 10];

    static GameView SampleGame() =>
      new()
      {
        User = Player("User", PlayerName.User,
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Backs/Back_Steampunk_Style_Color_1"),
        Opponent = Player("Opponent", PlayerName.Opponent,
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Backs/Back_Elf_Style_Color_1"),
        Arena = new ArenaView
        {
          IdentityAction = IdentityAction.InitiateRaid
        }
      };

    static CardId CardId(int id) => new() { Value = id };

    static GameObjectId CardObjectId(CardId cardId) => new() { CardId = cardId };

    static TimeValue TimeMs(int ms) => new() { Milliseconds = ms };

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

    static bool IsItem(int cardId) => cardId % 2 == 0;

    static CardView RevealedCard(
      int cardId,
      string title,
      string text,
      string image,
      int manaCost,
      CardTargeting? targeting = null,
      ObjectPosition? releasePosition = null)
    {
      var roomTarget = new CardTargeting
      {
        PickRoom = new PickRoom
        {
          ValidRooms =
          {
            RoomId.Crypts,
            RoomId.Sanctum,
            RoomId.Treasury
          }
        }
      };

      var roomPos = new ObjectPosition
      {
        Room = new ObjectPositionRoom
        {
          RoomLocation = RoomLocation.Front
        }
      };

      var itemPos = new ObjectPosition
      {
        Item = new ObjectPositionItem
        {
          ItemLocation = ItemLocation.Left
        }
      };

      return new CardView
      {
        CardId = CardId(cardId),
        CardBack = Sprite(
          "LittleSweetDaemon/TCG_Card_Fantasy_Design/Backs/Back_Steampunk_Style_Color_1"),
        CardIcons = new CardIcons
        {
          TopLeftIcon = new CardIcon
          {
            Background = Sprite("LittleSweetDaemon/TCG_Card_Fantasy_Design/Icons/Icon_Mana_Color_01"),
            Text = manaCost + ""
          }
        },
        RevealedCard = new RevealedCardView
        {
          CardFrame = Sprite(
            "LittleSweetDaemon/TCG_Card_Fantasy_Design/Cards/Card_Steampunk_Style_Color_1"),
          TitleBackground = Sprite(
            "LittleSweetDaemon/TCG_Card_Design/Magic_Card/Magic_Card_Face_Tape"),
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
          // Targeting = targeting,
          // OnReleasePosition = releasePosition ?? new CardPosition
          // {
          //   Item = new CardPositionItem
          //   {
          //     ItemLocation = ItemLocation.Right
          //   }
          // },
          Cost = new CardCost
          {
            CanPlay = false,
            CanPlayAlgorithm = CanPlayAlgorithm.Optimistic,
            ActionCost = 1,
            ManaCost = manaCost
          }
        }
      };
    }

    CardView OpponentCard(string title, int cardId, int image) => new()
    {
      CardId = CardId(cardId),
      CardBack = Sprite(
        "LittleSweetDaemon/TCG_Card_Fantasy_Design/Backs/Back_Elf_Style_Color_1"),
      CardIcons = new CardIcons
      {
        TopLeftIcon = new CardIcon
        {
          Background = Sprite("LittleSweetDaemon/TCG_Card_Fantasy_Design/Icons/Icon_Mana_Color_01"),
          Text = "4"
        }
      },
      OnCreatePosition = new ObjectPosition
      {
        Deck = new ObjectPositionDeck
        {
          Owner = PlayerName.Opponent
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
        }
      }
    };

    IEnumerator InitiateRaid(InitiateRaidAction action)
    {
      return _registry.CommandService.HandleCommands(new GameCommand
      {
        InitiateRaid = new InitiateRaidCommand
        {
          Initiator = PlayerName.User,
          RoomId = action.RoomId
        }
      }, new GameCommand
      {
        RenderInterface = new RenderInterfaceCommand
        {
          RaidControls = new InterfacePositionRaidControls
          {
            Node = action.RoomId switch
            {
              RoomId.Sanctum => SanctumRaidControls(),
              RoomId.Treasury => TreasuryRaidControls(),
              RoomId.Crypts => CryptsRaidControls(),
              _ => null
            }
          }
        }
      });
    }

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
      OptimisticUpdate = new CommandList
      {
        Commands =
        {
          FireProjectile(IdentityCardId(PlayerName.User), DiscardPileObjectId(PlayerName.Opponent), 4)
        }
      },
      Payload = Any.Pack(new CommandList
      {
        Commands =
        {
          ClearRaidControls(),
          RunInParallel(
            EndRaid(),
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
      }, Button("Continue", action: AccessHandAction(RoomId.Sanctum)));

    StandardAction AccessHandAction(RoomId fromRoom) => new()
    {
      OptimisticUpdate = new CommandList
      {
        Commands =
        {
          FireProjectile(CardObjectId(CardId(1)), IdentityCardId(PlayerName.User), 3)
        }
      },
      Payload = Any.Pack(new CommandList
      {
        Commands =
        {
          MoveToRoom(1, RoomId.Sanctum),
          FireProjectile(IdentityCardId(PlayerName.User), IdentityCardId(PlayerName.Opponent), 4),
          ClearRaidControls(),
          RunInParallel(
            MoveIdentityToContainer(PlayerName.User),
            MoveIdentityToContainer(PlayerName.Opponent)
          ),
          UpdateCard(OpponentCard("Revealed Card", 65539, 18)),
          UpdateCard(OpponentCard("Scheme Card", 65541, 19)),
          RunInParallel(_opponentHandCards.Select(id => MoveToBrowser(CardObjectId(id)))),
          RenderObjectButton(CardObjectId(CardId(65541)), "Score!", ScoreAction(65541,
            RunInParallel(_opponentHandCards
              .Except(CollectionUtils.Once(CardId(65541)))
              .Select(id => MoveToHand(id.Value, PlayerName.Opponent)))))
        }
      })
    };

    GameObjectId IdentityCardId(PlayerName playerName) => new()
    {
      CardId = new CardId
      {
        IdentityCard = playerName
      }
    };

    // ReSharper disable once UnusedMember.Local
    GameObjectId DeckObjectId(PlayerName playerName) => new()
    {
      Deck = playerName
    };

    // ReSharper disable once UnusedMember.Local
    GameObjectId HandObjectId(PlayerName playerName) => new()
    {
      Hand = playerName
    };

    GameObjectId DiscardPileObjectId(PlayerName playerName) => new()
    {
      DiscardPile = playerName
    };

    GameCommand UpdateCard(CardView cardView) => new()
    {
      UpdateCard = new UpdateCardCommand
      {
        Card = cardView
      }
    };

    Node TreasuryRaidControls() => Row("ControlButtons",
      new FlexStyle
      {
        JustifyContent = FlexJustify.FlexEnd,
        FlexGrow = 1,
        AlignItems = FlexAlign.Center,
        Wrap = FlexWrap.WrapReverse,
      },
      Button("The Maker's Eye\n5\uf06d", action: CardStrikeAction(RoomId.Treasury), smallText: true, orange: true),
      Button("Gordian Blade\n3\uf06d", action: null, smallText: true, orange: true),
      Button("Continue"));

    Node Button(string label, StandardAction? action = null, bool smallText = false, bool orange = false) => Row(
      "Button",
      new FlexStyle
      {
        Margin = AllDip(8),
        Height = Dip(88),
        MinWidth = Dip(132),
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

    StandardAction CardStrikeAction(RoomId fromRoom) =>
      new()
      {
        Payload = Any.Pack(AccessDeck()),
        OptimisticUpdate = new CommandList
        {
          Commands =
          {
            ClearRaidControls(),
            FireProjectile(
              CardObjectId(CardId(2)),
              CardObjectId(CardId(1)),
              projectileNumber: 8,
              additionalHit: true,
              hideOnHit: true,
              jumpToRoomOnHit: fromRoom)
          }
        }
      };

    GameCommand ClearRaidControls() =>
      new()
      {
        RenderInterface = new RenderInterfaceCommand
        {
          RaidControls = new InterfacePositionRaidControls()
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
            SourceId = CardObjectId(new CardId
            {
              IdentityCard = PlayerName.User
            }),
            TargetId = new GameObjectId
            {
              Deck = PlayerName.Opponent
            },
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
          CreateCard = new CreateCardCommand
          {
            Card = OpponentCard("Scheme", 55555, 92)
          }
        },
        MoveToRaidIndex(55555, 1),
        new GameCommand
        {
          CreateCard = new CreateCardCommand
          {
            Card = OpponentCard("Not A Scheme", 55556, 98)
          }
        },
        MoveToRaidIndex(55556, 1),
        MoveIdentityToContainer(PlayerName.User),
        RenderObjectButton(CardObjectId(CardId(55555)), "Score!",
          ScoreAction(55555, MoveToDeck(55556, PlayerName.Opponent)))
      }
    };

    GameCommand RenderObjectButton(GameObjectId id, string label, StandardAction onClick)
    {
      return new GameCommand
      {
        RenderInterface = new RenderInterfaceCommand
        {
          ObjectControls = new InterfacePositionObjectControls
          {
            ControlNodes =
            {
              new ObjectControlNode
              {
                GameObjectId = id,
                Node = Button(label, action: onClick, smallText: false, orange: true)
              }
            }
          }
        }
      };
    }

    GameCommand MoveIdentityToContainer(PlayerName playerName) =>
      new()
      {
        MoveGameObject = new MoveGameObjectCommand
        {
          Id = CardObjectId(new CardId
          {
            IdentityCard = playerName
          }),
          Position = new ObjectPosition
          {
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
        MoveGameObject = new MoveGameObjectCommand
        {
          Id = DiscardPileObjectId(playerName),
          Position = new ObjectPosition
          {
            DiscardPileContainer = new ObjectPositionDiscardPileContainer
            {
              Owner = playerName
            }
          }
        }
      };

    GameCommand MoveToRaidIndex(int cardId, int index) => MoveToRaidIndex(CardObjectId(CardId(cardId)), index);

    GameCommand MoveToRaidIndex(GameObjectId id, int index) => new()
    {
      MoveGameObject = new MoveGameObjectCommand
      {
        Id = id,
        Position = new ObjectPosition
        {
          Raid = new ObjectPositionRaid
          {
            Index = index
          }
        }
      }
    };

    GameCommand MoveToBrowser(GameObjectId id) => new()
    {
      MoveGameObject = new MoveGameObjectCommand
      {
        Id = id,
        Position = new ObjectPosition
        {
          Browser = new ObjectPositionBrowser()
        }
      }
    };

    StandardAction ScoreAction(int cardId, GameCommand cleanUp) => new()
    {
      Payload = Any.Pack(new CommandList
      {
        Commands =
        {
          MoveToDeckContainer(PlayerName.Opponent),
          MoveToIdentity(cardId),
          MoveToOffscreen(cardId),
          SetUserScore(2),
          EndRaid()
        }
      }),
      OptimisticUpdate = new CommandList
      {
        Commands =
        {
          new GameCommand
          {
            RenderInterface = new RenderInterfaceCommand
            {
              ObjectControls = new InterfacePositionObjectControls()
            }
          },
          MoveToScored(cardId),
          cleanUp,
          PlayHitEffect(cardId, 4, 700),
          PlayHitEffect(cardId, 4)
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

    GameCommand PlayHitEffect(int cardId, int i, int duration = 300) =>
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
            GameObject = CardObjectId(CardId(cardId))
          },
          Duration = TimeMs(duration),
          Scale = 2.0f
        }
      };

    // ReSharper disable once UnusedMember.Local
    GameCommand Delay(int ms) => new()
    {
      Delay = new DelayCommand
      {
        Duration = TimeMs(ms)
      }
    };

    GameCommand MoveToScored(int cardId) => new()
    {
      MoveGameObject = new MoveGameObjectCommand
      {
        Id = CardObjectId(CardId(cardId)),
        Position = new ObjectPosition
        {
          Scored = new ObjectPositionScored()
        }
      }
    };

    GameCommand MoveToIdentity(int cardId) => new()
    {
      MoveGameObject = new MoveGameObjectCommand
      {
        Id = CardObjectId(CardId(cardId)),
        Position = new ObjectPosition
        {
          Identity = new ObjectPositionIdentity
          {
            Owner = PlayerName.User
          }
        }
      }
    };

    GameCommand MoveToOffscreen(int cardId) => new()
    {
      MoveGameObject = new MoveGameObjectCommand
      {
        Id = CardObjectId(CardId(cardId)),
        Position = new ObjectPosition
        {
          Offscreen = new ObjectPositionOffscreen()
        }
      }
    };

    GameCommand SetUserScore(int score) => new()
    {
      RenderGame = new RenderGameCommand
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

    GameCommand EndRaid() => new()
    {
      EndRaid = new EndRaidCommand
      {
        Initiator = PlayerName.User
      }
    };

    GameCommand MoveToDeck(int cardId, PlayerName owner) => new()
    {
      MoveGameObject = new MoveGameObjectCommand
      {
        Id = CardObjectId(CardId(cardId)),
        Position = new ObjectPosition
        {
          Deck = new ObjectPositionDeck
          {
            Owner = owner
          }
        }
      }
    };

    GameCommand MoveToHand(int cardId, PlayerName owner) => new()
    {
      MoveGameObject = new MoveGameObjectCommand
      {
        Id = CardObjectId(CardId(cardId)),
        Position = new ObjectPosition
        {
          Hand = new ObjectPositionHand
          {
            Owner = owner
          }
        }
      }
    };


    GameCommand MoveToRoom(int cardId, RoomId roomId) => new()
    {
      MoveGameObject = new MoveGameObjectCommand
      {
        Id = CardObjectId(CardId(cardId)),
        Position = new ObjectPosition
        {
          Room = new ObjectPositionRoom
          {
            RoomId = roomId,
            RoomLocation = RoomLocation.Front
          }
        }
      }
    };

    GameCommand MoveToDeckContainer(PlayerName owner) => new()
    {
      MoveGameObject = new MoveGameObjectCommand
      {
        Id = new GameObjectId
        {
          Deck = owner
        },
        Position = new ObjectPosition
        {
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
      GameObjectId sourceId,
      GameObjectId targetId,
      int projectileNumber,
      bool additionalHit = false,
      bool hideOnHit = false,
      RoomId? jumpToRoomOnHit = null) => new()
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
            Room = new ObjectPositionRoom
            {
              RoomId = r,
              RoomLocation = RoomLocation.Front
            }
          }
          : null
      }
    };

    IEnumerator LevelUpRoom()
    {
      var updated = Card1.Clone();
      updated.CardIcons.ArenaIcon = new CardIcon
      {
        Text = "1",
        Background = Sprite("LittleSweetDaemon/TCG_Card_Elemental_Design/Number_Icons/Number_Icons_Color_3")
      };
      return _registry.CommandService.HandleCommands(new GameCommand
      {
        UpdateCard = new UpdateCardCommand
        {
          Card = updated
        }
      }, new GameCommand
      {
        MoveGameObject = new MoveGameObjectCommand
        {
          Id = IdentityCardId(PlayerName.User),
          Position = new ObjectPosition
          {
            IdentityContainer = new ObjectPositionIdentityContainer
            {
              Owner = PlayerName.User
            }
          }
        }
      });
    }
  }
}