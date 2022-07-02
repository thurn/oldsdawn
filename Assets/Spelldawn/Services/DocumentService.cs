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
    VisualElement _supplementalCardInfo = null!;
    Node _supplementalCardInfoNode = null!;

    public VisualElement RootVisualElement => _document.rootVisualElement;

    public void Initialize()
    {
      _document.rootVisualElement.Clear();
      AddRoot("Main Controls", out _mainControls, out _mainControlsNode);
      AddRoot("Card Controls", out _cardControls, out _cardControlsNode);
      AddRoot("SupplementalCardInfo", out _supplementalCardInfo, out _supplementalCardInfoNode);
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
        _registry.ActionService.HandleAction(new GameAction
        {
          FetchPanel = new FetchPanelAction
          {
            PanelAddress = address
          }
        });
      }
      else
      {
        _openPanels.Remove(address);
      }

      RenderPanels();
    }

    public bool IsOpen(PanelAddress address) => _openPanels.Contains(address);

    public bool IsAnyPanelOpen() => _openPanels.Count > 0;

    public void HandleUpdatePanels(UpdatePanelsCommand command)
    {
      foreach (var panel in command.Panels)
      {
        _panelCache[panel.Address] = panel.Node;
      }

      RenderPanels();
    }

    public void RenderMainControls(InterfaceMainControls? mainControls)
    {
        Reconcile(
          ref _mainControlsNode,
          ref _mainControls,
          MainControls(mainControls?.Node));

        Reconcile(
          ref _cardControlsNode,
          ref _cardControls,
          CardAnchors(mainControls?.CardAnchorNodes ?? Enumerable.Empty<CardAnchorNode>()));
    }

    void RenderPanels()
    {
      Reconcile(
        ref _fullScreenNode,
        ref _fullScreen,
        FullScreen(_openPanels.Select(p => _panelCache.GetValueOrDefault(p)).WhereNotNull()));
    }

    void Reconcile(ref Node previousNode, ref VisualElement previousElement, Node newNode)
    {
      var result = Reconciler.Update(_registry, newNode, previousElement, previousNode);

      if (result != null)
      {
        previousElement = result;
      }

      previousNode = newNode;
    }

    public void ClearSupplementalCardInfo()
    {
      Reconcile(
        ref _supplementalCardInfoNode,
        ref _supplementalCardInfo,
        new Node());
    }

    public void RenderSupplementalCardInfo(Card card, Node node, bool nodeLeft)
    {
      Reconcile(
        ref _supplementalCardInfoNode,
        ref _supplementalCardInfo,
        AnchorToCard(card, node, new List<CardAnchor>
        {
          new()
          {
            CardCorner = nodeLeft ? AnchorCorner.TopRight : AnchorCorner.TopLeft,
            NodeCorner = nodeLeft ? AnchorCorner.TopLeft : AnchorCorner.TopRight
          }
        }));
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

    Node CardAnchors(IEnumerable<CardAnchorNode> nodes)
    {
      return Row("CardAnchors", new FlexStyle
      {
        Position = FlexPosition.Absolute,
        Inset = AllDip(0),
      }, nodes.Select(RenderCardAnchorNode));
    }

    Node RenderCardAnchorNode(CardAnchorNode anchorNode) =>
      AnchorToCard(_registry.CardService.FindCard(anchorNode.CardId), anchorNode.Node, anchorNode.Anchors);

    Node AnchorToCard(Card card, Node node, IEnumerable<CardAnchor> anchors)
    {
      node.Style.Position = FlexPosition.Absolute;
      var inset = new DimensionGroup();

      foreach (var anchor in anchors)
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
  }
}