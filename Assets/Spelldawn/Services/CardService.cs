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
using DG.Tweening;
using Spelldawn.Battle;
using static Spelldawn.Masonry.MasonUtil;
using Spelldawn.Masonry;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEditor;
using UnityEngine;
using UnityEngine.UIElements;
using FlexDirection = UnityEngine.UIElements.FlexDirection;
using TimeValue = UnityEngine.UIElements.TimeValue;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class CardService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    VisualElement? _optimisticCard;

    public async void DrawOptimisticCard()
    {
      _optimisticCard?.RemoveFromHierarchy();

      var deck = _registry.DocumentService.Find(NodeNames.Deck);
      _optimisticCard = await Mason.Render(_registry, Column("OptimisticCard", new FlexStyle
      {
        Width = Dip(38),
        FlexShrink = 0,
        FixedBackgroundImageAspectRatio = true,
        BackgroundImage = Sprite("LittleSweetDaemon/TCG_Card_Design/Customized/ChampionCardBack"),
        Position = FlexPosition.Absolute,
        Inset = PositionDip(deck.worldBound.x + 6, deck.worldBound.y),
      }));

      _registry.GameDocument.rootVisualElement.Add(_optimisticCard);
      var target = _registry.DocumentService.Find(NodeNames.CardStaging);

      TweenUtils.ToLeft(_optimisticCard, target.worldBound.xMin, 1000f);
      TweenUtils.ToTop(_optimisticCard, target.worldBound.yMin, 1000f);
      TweenUtils.ToWidth(_optimisticCard, target.worldBound.width, 1000f);
      TweenUtils.ToHeight(_optimisticCard, target.worldBound.height, 1000f);

      var c3 = _registry.DocumentService.Find("Card3");
      c3.RemoveFromHierarchy();

      _container.Add(Child());
    }

    VisualElement _container;

    void Start()
    {
      _container = new VisualElement
      {
        style =
        {
          width = 400,
          height = 200,
          flexDirection = FlexDirection.Row,
          justifyContent = Justify.SpaceBetween,
          alignContent = Align.Center,
          backgroundColor = Color.gray,
          transitionProperty = new StyleList<StylePropertyName>(new List<StylePropertyName>
          {
            "all"
          }),
          transitionDelay = new StyleList<TimeValue>(new List<TimeValue> { new(1, TimeUnit.Second) })
        }
      };

      _container.Add(Child());
      _container.Add(Child());
      _container.Add(Child());

      _registry.GameDocument.rootVisualElement.Add(_container);
    }

    static VisualElement Child()
    {
      return new VisualElement
      {
        style =
        {
          width = 25,
          height = 25,
          marginTop = 25,
          marginLeft = 25,
          marginBottom = 25,
          marginRight = 25,
          backgroundColor = Color.red,
          transitionProperty = new StyleList<StylePropertyName>(new List<StylePropertyName>
          {
            "all"
          }),
          transitionDelay = new StyleList<TimeValue>(new List<TimeValue> { new(1, TimeUnit.Second) })
        }
      };
    }
  }
}