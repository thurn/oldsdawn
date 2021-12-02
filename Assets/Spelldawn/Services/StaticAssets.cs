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
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class StaticAssets : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;

    [SerializeField] AudioClip _drawCardStartSound = null!;
    public void PlayDrawCardStartSound() => Play(_drawCardStartSound);

    [SerializeField] AudioClip _drawCardSound = null!;
    public void PlayDrawCardSound() => Play(_drawCardSound);

    [SerializeField] AudioClip _addManaSound = null!;
    public void PlayAddManaSound() => Play(_addManaSound);

    [SerializeField] AudioClip _dawnSound = null!;
    public void PlayDawnSound() => Play(_dawnSound);

    [SerializeField] AudioClip _duskSound = null!;
    public void PlayDuskSound() => Play(_duskSound);

    [SerializeField] AudioClip _victorySound = null!;
    public void PlayVictorySound() => Play(_victorySound);

    [SerializeField] AudioClip _defeatSound = null!;
    public void PlayDefeatSound() => Play(_defeatSound);

    [SerializeField] List<AudioClip> _cardPlacementSounds = null!;
    public void PlayCardSound() => PlayRandomSound(_cardPlacementSounds);

    [SerializeField] List<AudioClip> _buttonClickSounds = null!;
    public void PlayButtonSound() => PlayRandomSound(_buttonClickSounds);

    [SerializeField] List<AudioClip> _magicWhooshes = null!;
    public void PlayWhooshSound() => PlayRandomSound(_magicWhooshes);

    [SerializeField] List<AudioClip> _fireProjectileSounds = null!;
    public void PlayFireProjectileSound() => PlayRandomSound(_fireProjectileSounds);

    [SerializeField] List<AudioClip> _impactSounds = null!;
    public void PlayImpactSound() => PlayRandomSound(_impactSounds);

    void Play(AudioClip clip)
    {
      _registry.MainAudioSource.PlayOneShot(clip);
    }

    void PlayRandomSound(List<AudioClip> list)
    {
      Play(list[Random.Range(0, list.Count)]);
    }
  }
}