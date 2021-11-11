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
using DG.Tweening;
using Spelldawn.Protos;
using Spelldawn.Services;
using Spelldawn.Utils;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class RewardChest : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] Transform _cardSpawnPosition = null!;
    [SerializeField] Transform _popUpPosition = null!;
    [SerializeField] ObjectDisplay _rewardBrowser = null!;
    [SerializeField] TimedEffect _appearEffect = null!;
    [SerializeField] GameObject _appearLight = null!;
    [SerializeField] GameObject _chest = null!;
    [SerializeField] float _duration;
    [SerializeField] GameObject _buildupGlow = null!;
    [SerializeField] AudioClip _buildUpSound = null!;
    [SerializeField] Animator _animator = null!;
    [SerializeField] float _openDelay;
    [SerializeField] AudioSource _audio = null!;
    [SerializeField] GameObject _openEffect = null!;
    [SerializeField] AudioClip _openSound = null!;
    [SerializeField] bool _canBeOpened;
    DisplayRewardsCommand? _currentCommand;

    static readonly int Open = Animator.StringToHash("Open");

    // ReSharper disable once UnusedMember.Local (Called by Animator)
    void OnOpened()
    {
      _buildupGlow.SetActive(false);
      _openEffect.SetActive(true);
      _audio.PlayOneShot(_openSound);
      if (_currentCommand != null)
      {
        StartCoroutine(DisplayCards(_currentCommand));
      }
    }

    IEnumerator DisplayCards(DisplayRewardsCommand command)
    {
      var cards = command.Rewards
        .Select(c => _registry.ObjectPositionService.CreateCard(c, GameContext.RewardBrowser, animate: false))
        .ToList();
      for (var i = 0; i < cards.Count; ++i)
      {
        var card = cards[i];
        card.transform.position = _cardSpawnPosition.position + new Vector3(i * 0.05f, 0, 0);
        card.transform.localScale = Vector3.one * 0.05f;
      }

      foreach (var card in cards)
      {
        yield return TweenUtils.Sequence("PopUpReward")
          .Insert(0, card.transform.DOMove(_popUpPosition.position, 0.3f))
          .Insert(0, card.transform.DOScale(0.1f * Vector3.one, 0.3f))
          .SetEase(Ease.InExpo)
          .WaitForCompletion();
        yield return _rewardBrowser.AddObject(card);
      }
    }

    IEnumerator OnMouseUpAsButton()
    {
      if (_canBeOpened)
      {
        _canBeOpened = false;
        _buildupGlow.SetActive(true);
        _audio.PlayOneShot(_buildUpSound);
        yield return new WaitForSecondsRealtime(_openDelay);
        _animator.SetTrigger(Open);
      }
    }

    public IEnumerator HandleDisplayRewards(DisplayRewardsCommand command)
    {
      gameObject.SetActive(true);
      yield return new WaitForSeconds(0.5f);
      _appearEffect.gameObject.SetActive(true);
      _appearLight.SetActive(true);
      yield return new WaitForSeconds(_duration);
      _chest.SetActive(true);
      _canBeOpened = true;
      _currentCommand = command;
    }
  }
}