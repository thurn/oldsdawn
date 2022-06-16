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
using Spelldawn.Protos;
using Spelldawn.Services;
using UnityEngine;

namespace Spelldawn.Tests
{
  public static class RunOverlordTestGame
  {
    public static IEnumerator Run(EndToEndTestService service)
    {
      Debug.Log("Running Overlord Test Game");
      service.Registry.GameService.CurrentGameId = null;
      service.Registry.GameService.PlayerId = new PlayerIdentifier { Value = 1 };
      service.Registry.ActionService.HandleAction(new GameAction
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
            },
            VsAgent = true
          }
        }
      });

      yield return service.WaitUntilSceneStart();
      service.Registry.GameService.Initialize(GlobalGameMode.Default);
      Debug.Log($"Run: Waiting Until Idle for Test Start");
      yield return service.WaitUntilIdle();
      yield return service.Capture("OpeningHand");
      service.StartStepping();

      service.ClickOn("Mulligan");
      yield return service.StepCapture("MulliganDraw");
      yield return service.StepCapture("DrawMulliganCard");
      yield return service.StepCapture("TurnStart", steps: 7);

      yield return service.Finish();
    }
  }
}