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
using Spelldawn.Game;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class AssetService : MonoBehaviour
  {
    readonly Dictionary<string, Object> _assets = new();

    public Sprite GetSprite(SpriteAddress address)
    {
      Errors.CheckNotNull(address, "Address is null");
      Errors.CheckArgument(_assets.ContainsKey(address.Address),
        $"Asset not found: {address}");
      return (Sprite)_assets[address.Address];
    }

    public void AssignSprite(SpriteRenderer spriteRenderer, SpriteAddress? address)
    {
      if (address != null)
      {
        spriteRenderer.sprite = GetSprite(address);
      }
    }

    public Font GetFont(FontAddress address)
    {
      Errors.CheckNotNull(address, "Address is null");
      Errors.CheckArgument(_assets.ContainsKey(address.Address), $"Asset not found: {address}");
      return (Font)_assets[address.Address];
    }

    public Projectile GetProjectile(ProjectileAddress address)
    {
      Errors.CheckNotNull(address, "Address is null");
      Errors.CheckArgument(_assets.ContainsKey(address.Address), $"Asset not found: {address}");
      return ComponentUtils.GetComponent<Projectile>((GameObject)_assets[address.Address]);
    }

    public TimedEffect GetEffect(EffectAddress address)
    {
      Errors.CheckNotNull(address, "Address is null");
      Errors.CheckArgument(_assets.ContainsKey(address.Address), $"Asset not found: {address}");
      return ComponentUtils.GetComponent<TimedEffect>((GameObject)_assets[address.Address]);
    }

    public AudioClip GetAudioClip(AudioClipAddress address)
    {
      Errors.CheckNotNull(address, "Address is null");
      return (AudioClip)_assets[address.Address];
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
          case GameCommand.CommandOneofCase.UpdateGameView:
            LoadGameAssets(requests, command.UpdateGameView.Game);
            break;
          case GameCommand.CommandOneofCase.InitiateRaid:
            break;
          case GameCommand.CommandOneofCase.CreateOrUpdateCard:
            LoadCardAssets(requests, command.CreateOrUpdateCard.Card);
            break;
          case GameCommand.CommandOneofCase.MoveGameObjects:
            break;
          case GameCommand.CommandOneofCase.PlaySound:
            LoadAudioClip(requests, command.PlaySound.Sound);
            break;
          case GameCommand.CommandOneofCase.FireProjectile:
            LoadProjectile(requests, command.FireProjectile.Projectile);
            LoadEffect(requests, command.FireProjectile.AdditionalHit);
            LoadAudioClip(requests, command.FireProjectile.FireSound);
            LoadAudioClip(requests, command.FireProjectile.ImpactSound);
            break;
          case GameCommand.CommandOneofCase.PlayEffect:
            LoadEffect(requests, command.PlayEffect.Effect);
            LoadAudioClip(requests, command.PlayEffect.Sound);
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
        case RenderInterfaceCommand.PositionOneofCase.MainControls:
          LoadNodeAssets(requests, command.MainControls.Node);
          break;
        case RenderInterfaceCommand.PositionOneofCase.CardAnchors:
          foreach (var controlNode in command.CardAnchors.AnchorNodes)
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
      }
    }

    void LoadPlayerAssets(IDictionary<string, ResourceRequest> requests, PlayerView? playerView)
    {
      if (playerView != null)
      {
        LoadPlayerInfoAssets(requests, playerView.PlayerInfo);
      }
    }

    void LoadCardListAssets(IDictionary<string, ResourceRequest> requests, IEnumerable<CardView>? cards)
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
        LoadSprite(requests, card.Image);
        LoadNodeAssets(requests, card.SupplementalInfo);
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
        LoadSprite(requests, playerInfo.CardBack);
        LoadSprite(requests, playerInfo.Portrait);
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

    void LoadAudioClip(IDictionary<string, ResourceRequest> requests, AudioClipAddress? address)
    {
      if (!string.IsNullOrWhiteSpace(address?.Address) && !_assets.ContainsKey(address.Address))
      {
        requests[address.Address] = Resources.LoadAsync<AudioClip>(address.Address);
      }
    }
  }
}