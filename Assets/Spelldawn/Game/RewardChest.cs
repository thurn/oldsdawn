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
using Spelldawn.Protos;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class RewardChest : MonoBehaviour
  {
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

    static readonly int Open = Animator.StringToHash("Open");

    // ReSharper disable once UnusedMember.Local (Called by Animator)
    void OnOpened()
    {
      _buildupGlow.SetActive(false);
      _openEffect.SetActive(true);
      _audio.PlayOneShot(_openSound);
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
      yield return new WaitForSeconds(0.5f);
      _appearEffect.gameObject.SetActive(true);
      _appearLight.SetActive(true);
      yield return new WaitForSeconds(_duration);
      _chest.SetActive(true);
      _canBeOpened = true;
    }
  }
}