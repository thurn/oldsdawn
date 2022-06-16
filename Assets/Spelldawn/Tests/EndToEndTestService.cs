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
using System.IO;
using Spelldawn.Masonry;
using Spelldawn.Protos;
using Spelldawn.Services;
using Spelldawn.Utils;
using UnityEngine;
using UnityEngine.UIElements;
using Directory = System.IO.Directory;

namespace Spelldawn.Tests
{
  /// <summary>Runs end-to-end screenshot tests.</summary>
  ///
  /// This originally used Unity's own screenshot testing tools, but I had a bunch of problems with them.
  public sealed class EndToEndTestService : MonoBehaviour
  {
    public Registry Registry { get; private set; } = null!;
    int _imageNumber = 1000;
    bool _sceneStart;
    string _directory = null!;
    
    [SerializeField] int _testStep = -1;
    [SerializeField] int _commandStep;

    public static EndToEndTestService Initialize(Registry registry, out bool runTests)
    {
      var testService = FindObjectOfType<EndToEndTestService>();
      if (testService)
      {
        testService.Registry = registry;
        testService.OnSceneStart();
        runTests = false;
        return testService;
      }
      else
      {
        var go = new GameObject("EndToEndTestService");
        var result = go.AddComponent<EndToEndTestService>();
        result.Registry = registry;
        result.OnCreated();
        result.OnSceneStart();
        runTests = true;
        return result;
      }
    }

    public void RunTests()
    {
      StartCoroutine(RunOverlordTestGame.Run(this));
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
      Registry.Graphy.SetActive(false);
      _sceneStart = true;
    }

    public void StartStepping()
    {
      _testStep = _commandStep;
    }

    public IEnumerator Capture(string imageName)
    {
      yield return new WaitForSeconds(0.3f);
      var path = Path.Combine(_directory, $"{_imageNumber++}_{imageName}.png");
      ScreenCapture.CaptureScreenshot(path);
    }

    public IEnumerator StepCapture(string imageName, int steps = 1)
    {
      var target = _commandStep + steps;
      Debug.Log($"StepCapture: Waiting for {target} to capture {imageName}");
      yield return new WaitUntil(() => _commandStep == target);
      yield return Capture(imageName);
      Debug.Log($"StepCapture Captured {imageName}: At {target}");
      _testStep = target;
    }

    public void Step()
    {
      _testStep++;
    }

    public IEnumerator WaitUntilSceneStart()
    {
      _sceneStart = false;
      yield return new WaitUntil(() => _sceneStart);
      yield return WaitUntilIdle();
    }

    public IEnumerator WaitUntilIdle()
    {
      yield return new WaitUntil(() => Registry.CommandService.Idle && Registry.ActionService.Idle);
      yield return new WaitForEndOfFrame();
    }

    public IEnumerator Finish()
    {
      yield return new WaitForSeconds(1.0f);
      Quit(0);
    }

    public IEnumerator OnCommandHandled(GameCommand command)
    {
      switch (command.CommandCase)
      {
        case GameCommand.CommandOneofCase.MoveGameObjects:
        case GameCommand.CommandOneofCase.MoveObjectsAtPosition:
        case GameCommand.CommandOneofCase.DisplayRewards:
        case GameCommand.CommandOneofCase.FireProjectile:
        case GameCommand.CommandOneofCase.PlayEffect:
          _commandStep += 1;
          break;
        case GameCommand.CommandOneofCase.RunInParallel:
          if (command.RunInParallel.Commands.Count > 0)
          {
            _commandStep += 1;
          }

          break;
      }

      if (_testStep >= 0)
      {
        Debug.Log($"OnCommandHandled: Waiting at {_commandStep}, test step is {_testStep}");
        yield return new WaitUntil(() => _testStep <= _commandStep);        
      }
    }

    public void ClickOn(string text)
    {
      var element = FindElementWithText(text);
      while (element != null)
      {
        if (element is INodeCallbacks c && c.Callbacks.Value.HasCallback(Callbacks.Event.Click))
        {
          c.Callbacks.Value.OnClick(new ClickEvent());
          return;
        }

        element = element.parent;
      }

      throw new InvalidOperationException($"Not found: {text}");
    }

    VisualElement? FindElementWithText(string text) =>
      FindText(Registry.DocumentService.RootVisualElement, text);

    VisualElement? FindText(VisualElement element, string text)
    {
      if (element is Label label && label.text.Contains(text))
      {
        return element;
      }

      foreach (var child in element.Children())
      {
        if (FindText(child, text) is { } result)
        {
          return result;
        }
      }

      return null;
    }

    public void Quit(int code)
    {
#if UNITY_EDITOR
      Debug.Log("Done running end-to-end tests");
#else
      Application.Quit(code);
#endif
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