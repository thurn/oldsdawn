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

using Spelldawn.Utils;
using UnityEngine;
using UnityEngine.SceneManagement;
using UnityEngine.UI;

#nullable enable

namespace Spelldawn.Menus
{
  public sealed class MainMenu : MonoBehaviour
  {
    [SerializeField] AudioSource _audioSource = null!;
    [SerializeField] AudioClip _click = null!;
    [SerializeField] RectTransform _mainMenu = null!;
    [SerializeField] RectTransform _settingsMenu = null!;
    [SerializeField] RectTransform _aboutMenu = null!;
    [SerializeField] Slider _musicVolume = null!;
    AsyncOperation? _sceneLoading;

    void Start()
    {
      if (!PlayerPrefs.HasKey(Preferences.MusicVolume))
      {
        PlayerPrefs.SetFloat(Preferences.MusicVolume, 0.25f);
      }

      _musicVolume.value = PlayerPrefs.GetFloat(Preferences.MusicVolume);
      _audioSource.volume = _musicVolume.value;
      var loadMusic = Resources.LoadAsync("OurMusicBox/OurMusicBox - Final War");
      loadMusic.completed += _ =>
      {
        _audioSource.clip = (AudioClip)loadMusic.asset;
        _audioSource.Play();
      };

      _sceneLoading = SceneManager.LoadSceneAsync("Labyrinth");
      _sceneLoading.allowSceneActivation = false;
    }

    public void OnPlayClicked()
    {
      PlayClick();
      if (_sceneLoading != null)
      {
        _sceneLoading.allowSceneActivation = true;
      }
    }

    public void OnSettingsClicked()
    {
      PlayClick();
      Clear();
      _settingsMenu.gameObject.SetActive(true);
    }

    public void OnAboutClicked()
    {
      PlayClick();
      Clear();
      _aboutMenu.gameObject.SetActive(true);
    }

    public void OnBackClicked()
    {
      PlayClick();
      Clear();
      _mainMenu.gameObject.SetActive(true);
    }

    public void OnSetMusicVolume()
    {
      PlayerPrefs.SetFloat(Preferences.MusicVolume, _musicVolume.value);
      _audioSource.volume = _musicVolume.value;
    }

    void PlayClick()
    {
      _audioSource.PlayOneShot(_click);
    }

    void Clear()
    {
      _mainMenu.gameObject.SetActive(false);
      _settingsMenu.gameObject.SetActive(false);
      _aboutMenu.gameObject.SetActive(false);
    }
  }
}