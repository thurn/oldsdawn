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
using UnityEngine.AddressableAssets;
using UnityEngine.ResourceManagement.AsyncOperations;
using Object = UnityEngine.Object;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class AssetService : MonoBehaviour
  {
    readonly Dictionary<string, Object> _assets = new();

    IEnumerator Start()
    {
      Debug.Log($"Start: Starting download");
      var startTime = Time.time;
      yield return Addressables.DownloadDependenciesAsync("Fonts/Roboto.ttf");
      Debug.Log($"Start: Done download in {Time.time - startTime} seconds");
    }
    
    public TResult Get<TResult>(string address) where TResult: Object
    {
      Errors.CheckNotNull(address, "Address is null");
      if (_assets.ContainsKey(address))
      {
        return (TResult)_assets[address];
      }
      else
      {
        Debug.LogError($"Asset not found: {address}");
        return null!;
      }
    }    

    public Sprite GetSprite(SpriteAddress address)
    {
      return Get<Sprite>(address.Address);
    }

    public void AssignSprite(SpriteRenderer spriteRenderer, SpriteAddress? address, float? referenceWidth = null)
    {
      if (address != null)
      {
        var sprite = GetSprite(address);
        if (referenceWidth != null)
        {
          spriteRenderer.transform.localScale = (referenceWidth.Value / sprite.texture.width) * Vector3.one;
        }

        spriteRenderer.sprite = sprite;
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
      var requests = new Dictionary<string, AsyncOperationHandle>();

      foreach (var command in commandList.Commands)
      {
        switch (command.CommandCase)
        {
          case GameCommand.CommandOneofCase.UpdatePanels:
            LoadUpdatePanelsAssets(requests, command.UpdatePanels);
            break;
          case GameCommand.CommandOneofCase.UpdateGameView:
            LoadGameAssets(requests, command.UpdateGameView.Game);
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
          case GameCommand.CommandOneofCase.CreateTokenCard:
            LoadCardListAssets(requests, new List<CardView> { command.CreateTokenCard.Card });
            break;
          case GameCommand.CommandOneofCase.None:
          default:
            break;
        }
      }

      yield return WaitForRequests(requests);
    }

    public IEnumerator LoadAssetsForNode(Node node)
    {
      var requests = new Dictionary<string, AsyncOperationHandle>();
      LoadNodeAssets(requests, node);
      yield return WaitForRequests(requests);
    }

    public IEnumerator WaitForRequests(IDictionary<string, AsyncOperationHandle> requests)
    {
      if (requests.Count > 0)
      {
        yield return new WaitUntil(() => requests.Values.All(r => r.Status == AsyncOperationStatus.Succeeded));

        foreach (var (address, request) in requests)
        {
          var result = request.Result as Object;
          if (result)
          {
            _assets[address] = result;
          }
          else
          {
            Debug.LogError($"Null asset for {address}");
          }
        }
      }    
    }

    void LoadUpdatePanelsAssets(IDictionary<string, AsyncOperationHandle> requests, UpdatePanelsCommand command)
    {
      foreach (var panel in command.Panels)
      {
        LoadNodeAssets(requests, panel.Node);
      }
    }

    void LoadInterfaceMainControlsAssets(IDictionary<string, AsyncOperationHandle> requests,
      InterfaceMainControls? mainControls)
    {
      if (mainControls != null)
      {
        LoadNodeAssets(requests, mainControls.Node);

        foreach (var controlNode in mainControls.CardAnchorNodes)
        {
          LoadNodeAssets(requests, controlNode.Node);
        }
      }
    }
    
    void LoadNodeAssets(IDictionary<string, AsyncOperationHandle> requests, Node? node)
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

    void LoadStyleAssets(IDictionary<string, AsyncOperationHandle> requests, FlexStyle? style)
    {
      if (style != null)
      {
        LoadBackground(requests, style.BackgroundImage);
        LoadFont(requests, style.Font);
      }
    }

    void LoadGameAssets(IDictionary<string, AsyncOperationHandle> requests, GameView? game)
    {
      if (game != null)
      {
        LoadPlayerAssets(requests, game.User);
        LoadPlayerAssets(requests, game.Opponent);
        LoadCardListAssets(requests, game.Cards);
        LoadInterfaceMainControlsAssets(requests, game.MainControls);        
      }
    }

    void LoadPlayerAssets(IDictionary<string, AsyncOperationHandle> requests, PlayerView? playerView)
    {
      if (playerView != null)
      {
        LoadPlayerInfoAssets(requests, playerView.PlayerInfo);
      }
    }

    void LoadCardListAssets(IDictionary<string, AsyncOperationHandle> requests, IEnumerable<CardView>? cards)
    {
      if (cards != null)
      {
        foreach (var card in cards)
        {
          LoadCardAssets(requests, card);
        }
      }
    }

    void LoadCardAssets(IDictionary<string, AsyncOperationHandle> requests, CardView? card)
    {
      if (card != null)
      {
        LoadCardIconsAssets(requests, card.CardIcons);
        LoadSprite(requests, card.ArenaFrame);
        LoadRevealedCardAssets(requests, card.RevealedCard);
      }
    }

    void LoadRevealedCardAssets(IDictionary<string, AsyncOperationHandle> requests, RevealedCardView? card)
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

    void LoadCardIconsAssets(IDictionary<string, AsyncOperationHandle> requests, CardIcons? cardIcons)
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

    void LoadCardIconAssets(IDictionary<string, AsyncOperationHandle> requests, CardIcon? cardIcon)
    {
      if (cardIcon != null)
      {
        LoadSprite(requests, cardIcon.Background);
      }
    }

    void LoadPlayerInfoAssets(IDictionary<string, AsyncOperationHandle> requests, PlayerInfo? playerInfo)
    {
      if (playerInfo != null)
      {
        LoadSprite(requests, playerInfo.CardBack);
        LoadSprite(requests, playerInfo.Portrait);
      }
    }
    
    void LoadBackground(IDictionary<string, AsyncOperationHandle> requests, NodeBackground? background)
    {
      if (background != null)
      {
        switch (background.BackgroundAddressCase)
        {
          case NodeBackground.BackgroundAddressOneofCase.Sprite:
            LoadSprite(requests, background.Sprite);
            break;
          case NodeBackground.BackgroundAddressOneofCase.RenderTexture:
            Load<RenderTexture>(requests, background.RenderTexture.Address);
            break;
        }
      }
    }

    void Load<T>(IDictionary<string, AsyncOperationHandle> requests, string? address) where T : Object
    {
      if (!string.IsNullOrWhiteSpace(address) && !_assets.ContainsKey(address))
      {
        requests[address] = Addressables.LoadAssetAsync<T>(address);
      }
    }    
    
    void LoadSprite(IDictionary<string, AsyncOperationHandle> requests, SpriteAddress? address)
    {
      if (!string.IsNullOrWhiteSpace(address?.Address) && !_assets.ContainsKey(address.Address))
      {
        requests[address.Address] = Addressables.LoadAssetAsync<Sprite>(address.Address);
      }
    }

    void LoadFont(IDictionary<string, AsyncOperationHandle> requests, FontAddress? address)
    {
      if (!string.IsNullOrWhiteSpace(address?.Address) && !_assets.ContainsKey(address.Address))
      {
        requests[address.Address] = Addressables.LoadAssetAsync<Font>(address.Address);
      }
    }

    void LoadProjectile(IDictionary<string, AsyncOperationHandle> requests, ProjectileAddress? address)
    {
      if (!string.IsNullOrWhiteSpace(address?.Address) && !_assets.ContainsKey(address.Address))
      {
        requests[address.Address] = Addressables.LoadAssetAsync<GameObject>(address.Address);
      }
    }

    void LoadEffect(IDictionary<string, AsyncOperationHandle> requests, EffectAddress? address)
    {
      if (!string.IsNullOrWhiteSpace(address?.Address) && !_assets.ContainsKey(address.Address))
      {
        requests[address.Address] = Addressables.LoadAssetAsync<GameObject>(address.Address);
      }
    }

    void LoadAudioClip(IDictionary<string, AsyncOperationHandle> requests, AudioClipAddress? address)
    {
      if (!string.IsNullOrWhiteSpace(address?.Address) && !_assets.ContainsKey(address.Address))
      {
        requests[address.Address] = Addressables.LoadAssetAsync<AudioClip>(address.Address);
      }
    }
  }
}