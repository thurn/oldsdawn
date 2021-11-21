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
using System.Linq;
using Spelldawn.Masonry;
using static Spelldawn.Masonry.MasonUtil;
using Spelldawn.Protos;
using UnityEngine;
using UnityEngine.UIElements;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class DocumentService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] UIDocument _document = null!;

    VisualElement _fullScreen = null!;
    VisualElement _raidControls = null!;
    VisualElement _cardControls = null!;

    void Start()
    {
      _document.rootVisualElement.Clear();
      AddRoot("Full Screen", out _fullScreen);
      AddRoot("Raid Controls", out _raidControls);
      AddRoot("Card Controls", out _cardControls);
    }

    float ScreenPxToElementDip(float value) => value * _document.panelSettings.referenceDpi / Screen.dpi;

    public Vector2 ScreenPositionToElementPosition(Vector3 screenPosition) =>
      new(ScreenPxToElementDip(screenPosition.x), ScreenPxToElementDip(Screen.height - screenPosition.y));

    public Vector2 TransformPositionToElementPosition(Transform t) =>
      ScreenPositionToElementPosition(_registry.MainCamera.WorldToScreenPoint(t.position));

    public void HandleRenderInterface(RenderInterfaceCommand command)
    {
      switch (command.PositionCase)
      {
        case RenderInterfaceCommand.PositionOneofCase.FullScreen:
          _fullScreen.Clear();
          _fullScreen.Add(Mason.Render(_registry, FullScreen(command.FullScreen.Node)));
          break;
        case RenderInterfaceCommand.PositionOneofCase.RaidControls:
          _raidControls.Clear();
          _raidControls.Add(Mason.Render(_registry, MainControls(command.RaidControls.Node)));
          break;
        case RenderInterfaceCommand.PositionOneofCase.ObjectControls:
          _cardControls.Clear();
          _cardControls.Add(Mason.Render(_registry,
            Row("ObjectControls", new FlexStyle(), command.ObjectControls.ControlNodes.Select(ObjectControl))));
          break;
        default:
          Debug.LogError($"Unknown interface position: {command.PositionCase}");
          goto case RenderInterfaceCommand.PositionOneofCase.FullScreen;
      }
    }

    public IEnumerator RenderMainControls(Node node)
    {
      return _registry.CommandService.HandleCommands(new GameCommand
      {
        RenderInterface = new RenderInterfaceCommand
        {
          RaidControls = new InterfacePositionRaidControls
          {
            Node = node
          }
        }
      });
    }

    void AddRoot(string elementName, out VisualElement element)
    {
      element = new VisualElement
      {
        name = elementName,
        style =
        {
          position = Position.Absolute,
          top = 0,
          right = 0,
          bottom = 0,
          left = 0
        },
        pickingMode = PickingMode.Ignore
      };
      _document.rootVisualElement.Add(element);
    }

    static Node FullScreen(Node? content) =>
      Row("FullScreen", new FlexStyle());

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

    Node ObjectControl(ObjectControlNode controlNode)
    {
      var position = TransformPositionToElementPosition(
        _registry.ObjectPositionService.Find(controlNode.GameObjectId).InterfaceAnchor());

      return Column("ObjectControl", new FlexStyle
      {
        Position = FlexPosition.Absolute,
        Inset = PositionDip(position.x - 250, position.y),
        Width = Dip(500),
        JustifyContent = FlexJustify.FlexStart,
        AlignItems = FlexAlign.Center
      }, controlNode.Node);
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
                    RaidControls = new InterfacePositionRaidControls()
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