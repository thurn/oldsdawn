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
using DG.Tweening;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEngine;
using Random = UnityEngine.Random;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class MusicService : MonoBehaviour
  {
    [SerializeField] MusicState _audioState = MusicState.Unspecified;
    [SerializeField] List<AudioClip> _gameplayTracks = null!;
    [SerializeField] AudioSource _gameplayAudioSource = null!;
    [SerializeField] List<AudioClip> _raidTracks = null!;
    [SerializeField] AudioSource _raidAudioSource = null!;
    AudioSource? _currentAudioSource;

    void Start()
    {
      SetMusicState(MusicState.Gameplay);

      _gameplayAudioSource.volume = PlayerPrefs.GetFloat(Preferences.MusicVolume);
      _raidAudioSource.volume = PlayerPrefs.GetFloat(Preferences.MusicVolume);
    }

    public void SetMusicState(MusicState state)
    {
      if (_audioState != state)
      {
        if (_currentAudioSource)
        {
          var source = _currentAudioSource!;
          TweenUtils
            .Sequence("FadeOutAudio")
            .Append(source.DOFade(0, 1.0f))
            .AppendCallback(() => source.Stop());
        }

        _audioState = state;
        if (state == MusicState.Silent)
        {
          _currentAudioSource = null;
          return;
        }

        var track = state switch
        {
          MusicState.Gameplay => _gameplayTracks[Random.Range(0, _gameplayTracks.Count)],
          MusicState.Raid => _raidTracks[Random.Range(0, _raidTracks.Count)],
          _ => throw new ArgumentOutOfRangeException(nameof(state), state, null)
        };

        _currentAudioSource = state switch
        {
          MusicState.Gameplay => _gameplayAudioSource,
          MusicState.Raid => _raidAudioSource,
          _ => throw new ArgumentOutOfRangeException(nameof(state), state, null)
        };

        _currentAudioSource.clip = track;
        _currentAudioSource.volume = 0f;
        _currentAudioSource.DOFade(PlayerPrefs.GetFloat(Preferences.MusicVolume), 1.0f);
        _currentAudioSource.Play();
      }
    }
  }
}