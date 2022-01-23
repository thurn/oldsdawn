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
using Spelldawn.Game;
using Spelldawn.Masonry;
using static Spelldawn.Masonry.MasonUtil;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEngine;
using UnityEngine.UIElements;

#nullable enable

namespace Spelldawn.Services
{
  public sealed record ElementPosition
  {
    public float Top { get; init; }
    public float Right { get; init; }
    public float Bottom { get; init; }
    public float Left { get; init; }
  }

  public sealed class DocumentService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] UIDocument _document = null!;
    readonly List<PanelAddress> _openPanels = new();
    readonly Dictionary<PanelAddress, Node> _panelCache = new();

    VisualElement _fullScreen = null!;
    Node _fullScreenNode = null!;
    VisualElement _mainControls = null!;
    Node _mainControlsNode = null!;
    VisualElement _cardControls = null!;
    Node _cardControlsNode = null!;

    void Start()
    {
      _document.rootVisualElement.Clear();
      AddRoot("Main Controls", out _mainControls, out _mainControlsNode);
      AddRoot("Card Controls", out _cardControls, out _cardControlsNode);
      AddRoot("Full Screen", out _fullScreen, out _fullScreenNode);
    }

    float ScreenPxToElementDip(float value) => value * _document.panelSettings.referenceDpi / Screen.dpi;

    /// <summary>
    /// Returns an ElementPosition in interface coordinates corresponding to a screen position.
    /// </summary>
    public ElementPosition ScreenPositionToElementPosition(Vector3 screenPosition) =>
      new()
      {
        Top = ScreenPxToElementDip(Screen.height - screenPosition.y),
        Right = ScreenPxToElementDip(Screen.width - screenPosition.x),
        Bottom = ScreenPxToElementDip(screenPosition.y),
        Left = ScreenPxToElementDip(screenPosition.x)
      };

    /// <summary>
    /// Returns an ElementPosition in interface coordinates corresponding to the position of the
    /// provided transform.
    /// </summary>
    public ElementPosition TransformPositionToElementPosition(Transform t)
      => ScreenPositionToElementPosition(_registry.MainCamera.WorldToScreenPoint(t.position));

    public void TogglePanel(bool open, PanelAddress address)
    {
      if (open)
      {
        _openPanels.Add(address);
      }
      else
      {
        _openPanels.Remove(address);
      }

      RenderPanels();
    }

    public bool IsOpen(PanelAddress address) => _openPanels.Contains(address);

    public void HandleRenderInterface(RenderInterfaceCommand command)
    {
      foreach (var panel in command.Panels)
      {
        _panelCache[panel.Address] = panel.Node;
      }

      RenderPanels();

      if (command.MainControls != null)
      {
        Reconcile(
          ref _mainControlsNode,
          ref _mainControls,
          MainControls(command.MainControls.Node));

        Reconcile(
          ref _cardControlsNode,
          ref _cardControls,
          Row("CardAnchors", new FlexStyle
          {
            Position = FlexPosition.Absolute,
            Inset = AllDip(0),
          }, command.MainControls.CardAnchorNodes.Select(RenderCardAnchorNode)));
      }
    }

    void RenderPanels()
    {
      Reconcile(
        ref _fullScreenNode,
        ref _fullScreen,
        FullScreen(_openPanels.Select(p => _panelCache.GetValueOrDefault(p)).WhereNotNull()));
    }

    void Reconcile(ref Node node, ref VisualElement element, Node newNode)
    {
      var result = Reconciler.Update(_registry, newNode, element, node);

      if (result != null)
      {
        element = result;
      }

      node = newNode;
    }


    public IEnumerator RenderMainControls(Node node)
    {
      return _registry.CommandService.HandleCommands(new GameCommand
      {
        RenderInterface = new RenderInterfaceCommand
        {
          MainControls = new InterfaceMainControls
          {
            Node = node
          }
        }
      });
    }

    public void ClearCardControls()
    {
      _cardControls.Clear();
    }

    public void RenderSupplementalCardInfo(Card card, Node node, CardNodeAnchorPosition position)
    {
      throw new NotImplementedException();
    }

    void AddRoot(string elementName, out VisualElement element, out Node node)
    {
      node = Row(elementName, new FlexStyle
      {
        Position = FlexPosition.Absolute,
        Inset = AllDip(0),
        PickingMode = FlexPickingMode.Ignore
      });
      element = Mason.Render(_registry, node);
      _document.rootVisualElement.Add(element);
    }

    static Node FullScreen(IEnumerable<Node> children) =>
      Row("FullScreen", new FlexStyle
      {
        Position = FlexPosition.Absolute,
        Inset = AllDip(0),
      }, children);

    static Node MainControls(Node? content) =>
      Row("MainControls", new FlexStyle
      {
        Position = FlexPosition.Absolute,
        Height = Dip(125),
        Inset = new DimensionGroup
        {
          Left = Dip(0),
          Right = Dip(0),
          Bottom = Dip(160)
        }
      }, content);

    Node RenderCardAnchorNode(CardAnchorNode anchorNode)
    {
      var card = _registry.CardService.FindCard(anchorNode.CardId);
      var node = anchorNode.Node;
      node.Style.Position = FlexPosition.Absolute;
      var inset = new DimensionGroup();

      foreach (var anchor in anchorNode.Anchors)
      {
        var target = anchor.CardCorner switch
        {
          AnchorCorner.TopLeft => card.TopLeftAnchor,
          AnchorCorner.TopRight => card.TopRightAnchor,
          AnchorCorner.BottomLeft => card.BottomLeftAnchor,
          AnchorCorner.BottomRight => card.BottomRightAnchor,
          _ => throw new ArgumentOutOfRangeException()
        };

        var position = TransformPositionToElementPosition(target);

        switch (anchor.NodeCorner)
        {
          case AnchorCorner.TopLeft:
            inset.Left = Dip(position.Left);
            inset.Top = Dip(position.Top);
            break;
          case AnchorCorner.TopRight:
            inset.Right = Dip(position.Right);
            inset.Top = Dip(position.Top);
            break;
          case AnchorCorner.BottomLeft:
            inset.Left = Dip(position.Left);
            inset.Bottom = Dip(position.Bottom);
            break;
          case AnchorCorner.BottomRight:
            inset.Right = Dip(position.Right);
            inset.Bottom = Dip(position.Bottom);
            break;
          default:
            throw new ArgumentOutOfRangeException();
        }
      }

      node.Style.Inset = inset;
      return node;
    }

    public static Node ControlGroup(params Node[] nodes) => Row("ControlGroup",
      new FlexStyle
      {
        JustifyContent = FlexJustify.Center,
        FlexGrow = 1,
        AlignItems = FlexAlign.Center,
        Wrap = FlexWrap.WrapReverse,
      },
      nodes);

    public static Node Button(string label, GameCommand? update, bool primary = false) => Row(
      $"Button {label}",
      new FlexStyle
      {
        Margin = AllDip(8),
        Height = Dip(88),
        MinWidth = Dip(132),
        JustifyContent = FlexJustify.Center,
        AlignItems = FlexAlign.Center,
        FlexShrink = 0,
        BackgroundImage = Sprite(primary
          ? "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/Buttons/Rescaled/Button_Orange"
          : "Poneti/ClassicFantasyRPG_UI/ARTWORKS/UIelements/Buttons/Rescaled/Button_Gray"),
        ImageSlice = ImageSlice(0, 16)
      },
      new EventHandlers
      {
        OnClick = new GameAction
        {
          StandardAction = new StandardAction
          {
            Update = new CommandList
            {
              Commands =
              {
                new GameCommand
                {
                  RenderInterface = new RenderInterfaceCommand
                  {
                    MainControls = new InterfaceMainControls()
                  }
                },
                update
              }
            }
          }
        }
      },
      Text(label, new FlexStyle
      {
        Margin = LeftRightDip(16),
        Padding = AllDip(0),
        Color = MakeColor(Color.white),
        FontSize = Dip(32),
        Font = Font("Fonts/Roboto"),
        TextAlign = TextAlign.MiddleCenter
      }));
  }
}