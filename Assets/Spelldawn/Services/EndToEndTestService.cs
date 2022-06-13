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

#nullable enable

using System.Collections;
using System.IO;
using Spelldawn.Protos;
using Spelldawn.Utils;
using UnityEditor;
using UnityEngine;
using Directory = System.IO.Directory;

namespace Spelldawn.Services
{
  /// <summary>Runs end-to-end screenshot tests.</summary>
  ///
  /// This originally used Unity's own screenshot testing tools, but I had a bunch of problems with them.
  public sealed class EndToEndTestService : MonoBehaviour
  {
    Registry _registry = null!;
    int _imageNumber = 1000;
    bool _sceneStart;
    string _directory = null!;

    public static void Initialize(Registry registry)
    {
      var testService = FindObjectOfType<EndToEndTestService>();
      if (testService)
      {
        testService._registry = registry;
        testService.OnSceneStart();
      }
      else
      {
        var go = new GameObject("EndToEndTestService");
        var result = go.AddComponent<EndToEndTestService>();
        result._registry = registry;
        result.OnCreated();
        result.OnSceneStart();
        result.StartCoroutine(result.Run());
      }
    }

    void OnCreated()
    {
      DontDestroyOnLoad(gameObject);
      Application.logMessageReceived += HandleException;
      PlayerPrefs.DeleteAll();
      PlayerPrefs.SetInt(Preferences.OfflineMode, 1);
      PlayerPrefs.SetInt(Preferences.InMemory, 1);
      _directory = Application.isEditor
        ? "/tmp/spelldawn/Screenshots"
        : Path.Combine(Application.dataPath, "Screenshots");
      Directory.CreateDirectory(_directory);
      Debug.Log($"Saving screenshots to {_directory}");
    }

    void OnSceneStart()
    {
      _registry.ManaDisplayForPlayer(PlayerName.User).DisableAnimation();
      _registry.ManaDisplayForPlayer(PlayerName.Opponent).DisableAnimation();
      _registry.ActionDisplayForPlayer(PlayerName.User).DisableAnimation();
      _registry.ActionDisplayForPlayer(PlayerName.Opponent).DisableAnimation();
      _registry.Graphy.SetActive(false);
      _sceneStart = true;
    }

    IEnumerator Run()
    {
      Debug.Log("Running End to End Tests");
      _registry.GameService.CurrentGameId = null;
      _registry.GameService.PlayerId = new PlayerIdentifier { Value = 1 };
      _registry.ActionService.HandleAction(new GameAction
      {
        CreateNewGame = new CreateNewGameAction
        {
          Side = PlayerSide.Overlord,
          OpponentId = new PlayerIdentifier
          {
            Value = 2
          },
          DebugOptions = new CreateGameDebugOptions
          {
            Deterministic = true,
            OverrideGameIdentifier = new GameIdentifier
            {
              Value = 0
            }
          }
        }
      });

      yield return WaitUntilSceneStart();
      _registry.GameService.Initialize(GlobalGameMode.Default);
      yield return WaitUntilIdle();
      Capture("OpeningHand");

      yield return Finish();
    }

    void Capture(string imageName)
    {
      var path = Path.Combine(_directory, $"{_imageNumber++}_{imageName}.png");
      ScreenCapture.CaptureScreenshot(path);
    }

    IEnumerator WaitUntilSceneStart()
    {
      _sceneStart = false;
      yield return new WaitUntil(() => _sceneStart);
      yield return WaitUntilIdle();
    }

    IEnumerator WaitUntilIdle()
    {
      yield return new WaitUntil(() => _registry.CommandService.Idle && _registry.ActionService.Idle);
      yield return new WaitForEndOfFrame();
    }

    IEnumerator Finish()
    {
      Debug.Log("Done Running End To End Tests");
      yield return new WaitForSeconds(1.0f);
      Quit(0);
    }

    void HandleException(string logString, string stackTrace, LogType type)
    {
      if (type is LogType.Exception)
      {
        Quit(1);
      }
    }

    static void Quit(int code)
    {
#if !UNITY_EDITOR
      Application.Quit(code);
#endif
    }
  }
}