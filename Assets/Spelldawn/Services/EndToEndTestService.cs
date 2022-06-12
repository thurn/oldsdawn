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
using System.Linq;
using Spelldawn.Protos;
using UnityEngine;
using Directory = System.IO.Directory;

namespace Spelldawn.Services
{
  /// <summary>Runs end-to-end screenshot tests.</summary>
  ///
  /// This originally used Unity's own screenshot testing tools, but I had a bunch of problems with them.
  public sealed class EndToEndTestService : MonoBehaviour
  {
    int _imageNumber = 1000;
    [SerializeField] Registry _registry = null!;
    
    public void Initialize()
    {
      Application.logMessageReceived += HandleException;      
      _registry.ManaDisplayForPlayer(PlayerName.User).DisableSymbolAnimation();
      _registry.ManaDisplayForPlayer(PlayerName.Opponent).DisableSymbolAnimation();
      _registry.ActionDisplayForPlayer(PlayerName.User).DisableSymbolAnimation();
      _registry.ActionDisplayForPlayer(PlayerName.Opponent).DisableSymbolAnimation();
      _registry.Graphy.SetActive(false);

      var args = System.Environment.GetCommandLineArgs();
      if (args.Any(arg => arg.Contains("test")))
      {
        StartCoroutine(Run());
      }
    }

    IEnumerator Run()
    {
      Debug.Log("Running End to End Tests");
      yield return new WaitForSeconds(5f);
      Capture("Start");
      yield return new WaitForSeconds(5f);
      Application.Quit();
    }

    void Capture(string imageName)
    {
      var dir = Path.Combine(Application.dataPath, "Screenshots");
      Directory.CreateDirectory(dir);
      var path = Path.Combine(dir, $"{_imageNumber++}_{imageName}.png");
      ScreenCapture.CaptureScreenshot(path);
    }
    
    void HandleException(string logString, string stackTrace, LogType type)
    {
      if (type is LogType.Exception)
      {
        Application.Quit(1);
      }
    }
  }
}