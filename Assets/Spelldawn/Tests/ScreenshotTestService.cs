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

using System;
using System.Collections;
using System.Collections.Generic;
using System.IO;
using DG.Tweening;
using Spelldawn.Protos;
using Spelldawn.Services;
using Spelldawn.Utils;
using UnityEngine;
using UnityEngine.SceneManagement;
using Directory = System.IO.Directory;

namespace Spelldawn.Tests
{
  /// <summary>Runs screenshot tests.</summary>
  ///
  /// This originally used Unity's own screenshot testing tools, but I had a bunch of problems with them.
  public sealed class ScreenshotTestService : MonoBehaviour
  {
    // Set a filename here to pause the test before and after it runs
    static readonly string? DebugPauseOn = null;
    
    public Registry Registry { get; private set; } = null!;
    string _directory = null!;
    readonly List<Sequence> _sequences = new();
    bool _sceneLoaded;

    public static ScreenshotTestService Initialize(Registry registry, out bool runTests)
    {
      var testService = FindObjectOfType<ScreenshotTestService>();
      if (testService)
      {
        testService.Registry = registry;
        testService.OnSceneStart();
        runTests = false;
        return testService;
      }
      else
      {
        var go = new GameObject("ScreenshotTestService");
        var result = go.AddComponent<ScreenshotTestService>();
        result.Registry = registry;
        result.OnCreated();
        result.OnSceneStart();
        runTests = true;
        return result;
      }
    }

    public void RunTests()
    {
      StartCoroutine(RunAsync());
    }

    public void OnAnimationStarted(Sequence sequence)
    {
      _sequences.Add(sequence);
    }
    
    IEnumerator RunAsync()
    {
      foreach (var asset in Resources.LoadAll<TextAsset>("TestRecordings"))
      {
        yield return RunTest(asset);
        yield return WaitForAnimations();
        _sceneLoaded = false;
        SceneManager.LoadSceneAsync("Labyrinth");
        yield return new WaitUntil(() => _sceneLoaded);
        _sequences.Clear();
        yield return new WaitForEndOfFrame();
      }
      
      yield return Finish();
    }

    IEnumerator RunTest(TextAsset textAsset)
    {
      var list = CommandList.Parser.ParseDelimitedFrom(new MemoryStream(textAsset.bytes));

      var count = 100;
      foreach (var command in list.Commands)
      {
        var fileName = $"{textAsset.name}_{count}.png";
        if (ShouldHandle(command.CommandCase) && ShouldPause(fileName))
        {
          Debug.Log($"Preparing to run {fileName}");
          Debug.Break();
        }
        yield return Registry.CommandService.HandleCommands(command);
        yield return WaitForAnimations();

        if (ShouldHandle(command.CommandCase))
        {
          if (ShouldPause(fileName))
          {
            Debug.Log($"Saving Screenshot for {fileName}");
            Debug.Break();
          }
          yield return Capture(fileName);
          count++;
        }
      }
    }

    void OnCreated()
    {
      DontDestroyOnLoad(gameObject);
      Application.logMessageReceived += HandleException;

      if (Math.Abs(Screen.dpi - 255.0) > 0.1f && !Application.isEditor)
      {
        Debug.LogError($"ERROR: DPI is {Screen.dpi}, screenshot tests can only run on a 255 dpi screen.");
        Quit(1);
        return;
      }

      PlayerPrefs.DeleteAll();
      PlayerPrefs.SetInt(Preferences.InMemory, 1);
      _directory = Application.isEditor
        ? "/tmp/spelldawn/Screenshots"
        : Path.Combine(Application.dataPath, "Screenshots");
      Debug.Log($"Saving screenshots to {_directory}");

      if (Directory.Exists(_directory))
      {
        Directory.Delete(_directory, recursive: true);
      }
      Directory.CreateDirectory(_directory);
    }

    void OnSceneStart()
    {
      Registry.ManaDisplayForPlayer(PlayerName.User).DisableAnimation();
      Registry.ManaDisplayForPlayer(PlayerName.Opponent).DisableAnimation();
      Registry.ActionDisplayForPlayer(PlayerName.User).DisableAnimation();
      Registry.ActionDisplayForPlayer(PlayerName.Opponent).DisableAnimation();
      Registry.IdentityCardForPlayer(PlayerName.User).DisableAnimation();
      Registry.IdentityCardForPlayer(PlayerName.Opponent).DisableAnimation();
      Registry.Graphy.SetActive(false);
      TweenUtils.EndToEndTests = this;
      _sceneLoaded = true;
    }
    
    IEnumerator Capture(string imageName)
    {
      yield return new WaitForSeconds(0.3f);
      var path = Path.Combine(_directory, imageName);
      ScreenCapture.CaptureScreenshot(path);
    }

    IEnumerator WaitForAnimations()
    {
      foreach (var sequence in _sequences)
      {
        if (sequence.IsActive())
        {
          yield return sequence.WaitForCompletion();
        }
      }
      
      _sequences.Clear();

      foreach (var system in FindObjectsOfType<ParticleSystem>())
      {
        if (system.isPlaying && !system.main.loop)
        {
          yield return new WaitUntil(() => !system.isPlaying);
        }
      }
      
      yield return new WaitForEndOfFrame();
    }

    IEnumerator Finish()
    {
      yield return new WaitForSeconds(1.0f);
      Quit(0);
    }
    
    static bool ShouldHandle(GameCommand.CommandOneofCase commandCase) => commandCase switch
    {
      GameCommand.CommandOneofCase.UpdateGameView => true,
      GameCommand.CommandOneofCase.MoveGameObjects => true,
      _ => false
    };

    static bool ShouldPause(string fileName) => DebugPauseOn is { } p && p == fileName;    

    void Quit(int code)
    {
      Registry.GameService.CurrentGameId = null;
      Application.Quit(code);
    }

    void HandleException(string logString, string stackTrace, LogType type)
    {
      if (type is LogType.Exception)
      {
        Quit(1);
      }
    }
  }
}