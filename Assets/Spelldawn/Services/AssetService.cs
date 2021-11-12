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
using Spelldawn.Game;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEngine;
using Object = UnityEngine.Object;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class AssetService : MonoBehaviour
  {
    readonly Dictionary<string, Object> _assets = new();

    public Sprite GetSprite(SpriteAddress address)
    {
      Errors.CheckArgument(_assets.ContainsKey(address.Address), $"Asset not found: {address}");
      return (Sprite)_assets[address.Address];
    }

    public Font GetFont(FontAddress address)
    {
      Errors.CheckArgument(_assets.ContainsKey(address.Address), $"Asset not found: {address}");
      return (Font)_assets[address.Address];
    }

    public Projectile GetProjectile(ProjectileAddress address)
    {
      Errors.CheckArgument(_assets.ContainsKey(address.Address), $"Asset not found: {address}");
      return ComponentUtils.GetComponent<Projectile>((GameObject)_assets[address.Address]);
    }

    public TimedEffect GetEffect(EffectAddress address)
    {
      Errors.CheckArgument(_assets.ContainsKey(address.Address), $"Asset not found: {address}");
      return ComponentUtils.GetComponent<TimedEffect>((GameObject)_assets[address.Address]);
    }

    public IEnumerator LoadAssets(CommandList commandList)
    {
      var requests = new Dictionary<string, ResourceRequest>();

      foreach (var command in commandList.Commands)
      {
        switch (command.CommandCase)
        {
          case GameCommand.CommandOneofCase.RenderInterface:
            LoadRenderInterfaceAssets(requests, command.RenderInterface);
            break;
          case GameCommand.CommandOneofCase.RenderGame:
            LoadGameAssets(requests, command.RenderGame.Game);
            break;
          case GameCommand.CommandOneofCase.InitiateRaid:
            break;
          case GameCommand.CommandOneofCase.CreateCard:
            LoadCardAssets(requests, command.CreateCard.Card);
            break;
          case GameCommand.CommandOneofCase.UpdateCard:
            LoadCardAssets(requests, command.UpdateCard.Card);
            break;
          case GameCommand.CommandOneofCase.MoveGameObjects:
            break;
          case GameCommand.CommandOneofCase.DestroyCard:
            break;
          case GameCommand.CommandOneofCase.UpdatePlayerState:
            LoadPlayerInfoAssets(requests, command.UpdatePlayerState.Info);
            LoadScoreViewAssets(requests, command.UpdatePlayerState.Score);
            break;
          case GameCommand.CommandOneofCase.FireProjectile:
            LoadProjectile(requests, command.FireProjectile.Projectile);
            LoadEffect(requests, command.FireProjectile.AdditionalHit);
            break;
          case GameCommand.CommandOneofCase.PlayEffect:
            LoadEffect(requests, command.PlayEffect.Effect);
            break;
          case GameCommand.CommandOneofCase.DisplayRewards:
            LoadCardListAssets(requests, command.DisplayRewards.Rewards);
            break;
          case GameCommand.CommandOneofCase.None:
          default:
            break;
        }
      }

      if (requests.Count > 0)
      {
        yield return new WaitUntil(() => requests.Values.All(r => r.isDone));

        foreach (var (address, request) in requests)
        {
          if (request.asset)
          {
            _assets[address] = request.asset;
          }
          else
          {
            Debug.LogError($"Null asset for {address}");
          }
        }
      }
    }

    void LoadRenderInterfaceAssets(IDictionary<string, ResourceRequest> requests, RenderInterfaceCommand command)
    {
      switch (command.PositionCase)
      {
        case RenderInterfaceCommand.PositionOneofCase.FullScreen:
          LoadNodeAssets(requests, command.FullScreen.Node);
          break;
        case RenderInterfaceCommand.PositionOneofCase.RaidControls:
          LoadNodeAssets(requests, command.RaidControls.Node);
          break;
        case RenderInterfaceCommand.PositionOneofCase.ObjectControls:
          foreach (var controlNode in command.ObjectControls.ControlNodes)
          {
            LoadNodeAssets(requests, controlNode.Node);
          }
          break;
      }
    }

    void LoadNodeAssets(IDictionary<string, ResourceRequest> requests, Node? node)
    {
      if (node != null)
      {
        LoadStyleAssets(requests, node.Style);
        LoadStyleAssets(requests, node.HoverStyle);
        LoadStyleAssets(requests, node.PressedStyle);

        foreach (var child in node.Children)
        {
          LoadNodeAssets(requests, child);
        }
      }
    }

    void LoadStyleAssets(IDictionary<string, ResourceRequest> requests, FlexStyle? style)
    {
      if (style != null)
      {
        LoadSprite(requests, style.BackgroundImage);
        LoadFont(requests, style.Font);
      }
    }

    void LoadGameAssets(IDictionary<string, ResourceRequest> requests, GameView? game)
    {
      if (game != null)
      {
        LoadPlayerAssets(requests, game.User);
        LoadPlayerAssets(requests, game.Opponent);
        LoadArenaAssets(requests, game.Arena);
      }
    }

    void LoadPlayerAssets(IDictionary<string, ResourceRequest> requests, PlayerView? playerView)
    {
      if (playerView != null)
      {
        LoadPlayerInfoAssets(requests, playerView.PlayerInfo);
        LoadScoreViewAssets(requests, playerView.Score);
        LoadCardListAssets(requests, playerView.Hand?.Cards);
        LoadCardListAssets(requests, playerView.DiscardPile?.Cards);
      }
    }

    void LoadArenaAssets(IDictionary<string, ResourceRequest> requests, ArenaView? arenaView)
    {
      if (arenaView != null)
      {
        foreach (var room in arenaView.Rooms)
        {
          LoadCardListAssets(requests, room.Cards);
        }

        LoadCardListAssets(requests, arenaView.Items);
      }
    }

    void LoadCardListAssets(IDictionary<string, ResourceRequest> requests, RepeatedField<CardView>? cards)
    {
      if (cards != null)
      {
        foreach (var card in cards)
        {
          LoadCardAssets(requests, card);
        }
      }
    }

    void LoadCardAssets(IDictionary<string, ResourceRequest> requests, CardView? card)
    {
      if (card != null)
      {
        LoadSprite(requests, card.CardBack);
        LoadCardIconsAssets(requests, card.CardIcons);
        LoadSprite(requests, card.ArenaFrame);
        LoadRevealedCardAssets(requests, card.RevealedCard);
      }
    }

    void LoadRevealedCardAssets(IDictionary<string, ResourceRequest> requests, RevealedCardView? card)
    {
      if (card != null)
      {
        LoadSprite(requests, card.CardFrame);
        LoadSprite(requests, card.TitleBackground);
        LoadSprite(requests, card.Jewel);
        LoadSprite(requests, card.ImageBackground);
        LoadSprite(requests, card.Image);
      }
    }

    void LoadCardIconsAssets(IDictionary<string, ResourceRequest> requests, CardIcons? cardIcons)
    {
      if (cardIcons != null)
      {
        LoadCardIconAssets(requests, cardIcons.TopLeftIcon);
        LoadCardIconAssets(requests, cardIcons.TopRightIcon);
        LoadCardIconAssets(requests, cardIcons.BottomRightIcon);
        LoadCardIconAssets(requests, cardIcons.BottomLeftIcon);
        LoadCardIconAssets(requests, cardIcons.ArenaIcon);
      }
    }

    void LoadCardIconAssets(IDictionary<string, ResourceRequest> requests, CardIcon? cardIcon)
    {
      if (cardIcon != null)
      {
        LoadSprite(requests, cardIcon.Background);
      }
    }

    void LoadPlayerInfoAssets(IDictionary<string, ResourceRequest> requests, PlayerInfo? playerInfo)
    {
      if (playerInfo != null)
      {
        LoadSprite(requests, playerInfo.Portrait);
        LoadCardAssets(requests, playerInfo.IdentityCard);
      }
    }

    void LoadScoreViewAssets(IDictionary<string, ResourceRequest> requests, ScoreView? scoreView)
    {
      if (scoreView != null)
      {
        LoadCardListAssets(requests, scoreView.ScoredCards);
      }
    }

    void LoadSprite(IDictionary<string, ResourceRequest> requests, SpriteAddress? address)
    {
      if (!string.IsNullOrWhiteSpace(address?.Address) && !_assets.ContainsKey(address.Address))
      {
        requests[address.Address] = Resources.LoadAsync<Sprite>(address.Address);
      }
    }

    void LoadFont(IDictionary<string, ResourceRequest> requests, FontAddress? address)
    {
      if (!string.IsNullOrWhiteSpace(address?.Address) && !_assets.ContainsKey(address.Address))
      {
        requests[address.Address] = Resources.LoadAsync<Font>(address.Address);
      }
    }

    void LoadProjectile(IDictionary<string, ResourceRequest> requests, ProjectileAddress? address)
    {
      if (!string.IsNullOrWhiteSpace(address?.Address) && !_assets.ContainsKey(address.Address))
      {
        requests[address.Address] = Resources.LoadAsync<GameObject>(address.Address);
      }
    }

    void LoadEffect(IDictionary<string, ResourceRequest> requests, EffectAddress? address)
    {
      if (!string.IsNullOrWhiteSpace(address?.Address) && !_assets.ContainsKey(address.Address))
      {
        requests[address.Address] = Resources.LoadAsync<GameObject>(address.Address);
      }
    }
  }
}