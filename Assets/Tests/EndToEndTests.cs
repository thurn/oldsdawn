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
using Spelldawn.Protos;
using Spelldawn.Services;
using Spelldawn.Utils;
using UnityEditor.SceneManagement;
using UnityEngine;
using UnityEngine.SceneManagement;
using UnityEngine.TestTools;
using UnityEngine.TestTools.Graphics;
using Object = UnityEngine.Object;

public class EndToEndTests
{
    Registry Registry
    {
        get
        {
            var registries = Object.FindObjectsOfType<Registry>();
            Debug.Assert(registries.Length == 1);
            return registries[0];            
        }
    }

    bool _sceneLoaded;
    
    [UnityTest]
    public IEnumerator RunTestGame()
    {
        SceneManager.sceneLoaded += OnSceneLoaded;
        PlayerPrefs.DeleteAll();
        PlayerPrefs.SetInt(Preferences.OfflineMode, 1);
        PlayerPrefs.SetInt(Preferences.InMemory, 1);
        
        yield return WaitUntilSceneLoaded(() =>
        {
            EditorSceneManager.LoadSceneAsyncInPlayMode(
                "Assets/Scenes/Labyrinth.unity",
                new LoadSceneParameters(LoadSceneMode.Additive));
        });
        
        Registry.GameService.CurrentGameId = null;
        Registry.GameService.PlayerId = new PlayerIdentifier { Value = 1 };
        
        yield return WaitUntilSceneLoaded(() =>
        {
            Registry.ActionService.HandleAction(new GameAction
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
        });

        yield return WaitUntilIdle();
        Debug.Assert(Registry.CardBrowser.AllObjects.Count == 5);
        
        yield return EndTest();
    }

    IEnumerator WaitUntilSceneLoaded(Action action)
    {
        _sceneLoaded = false;
        action();
        return new WaitUntil(() => _sceneLoaded);
    }

    IEnumerator WaitUntilIdle()
    {
        yield return new WaitUntil(() => Registry.CommandService.Idle && Registry.ActionService.Idle);
        
        // I was using WaitForEndOfFrame() for this but it just hangs forever when you run from the command line?!
        yield return new WaitForSeconds(0.01f);
    }

    IEnumerator EndTest()
    {
        // It's helpful to wait for the end of frame after tests to let cleanup code (e.g. in DOTween) run 
        yield return new WaitForSeconds(0.01f);
    }

    void OnSceneLoaded(Scene scene, LoadSceneMode mode)
    {
        Debug.Log($"OnSceneLoaded: {scene.name}");
        _sceneLoaded = true;
    }
}
