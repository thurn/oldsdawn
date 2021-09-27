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
using Spelldawn.Protos;
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class ActionService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;

    public void HandleAction(GameAction action)
    {
      StartCoroutine(HandleActionAsync(action));
    }

    IEnumerator<YieldInstruction> HandleActionAsync(GameAction action)
    {
      ApplyOptimisticResponse(action);

      // Send to server
      yield return new WaitForSeconds(Random.Range(0.1f, 1.5f));

      FakeActionResponse(action);
    }

    void ApplyOptimisticResponse(GameAction action)
    {
      switch (action.ActionCase)
      {
        case GameAction.ActionOneofCase.DebugLog:
          Debug.Log(action.DebugLog.Log);
          break;
        case GameAction.ActionOneofCase.DrawCard:
          _registry.CardService.DrawOptimisticCard();
          break;
      }
    }

    void FakeActionResponse(GameAction action)
    {
      switch (action.ActionCase)
      {
        case GameAction.ActionOneofCase.DrawCard:
          break;
      }
    }
  }
}