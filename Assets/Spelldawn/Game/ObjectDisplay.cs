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

using System.Collections;
using System.Collections.Generic;
using DG.Tweening;
using Spelldawn.Utils;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public abstract class ObjectDisplay : MonoBehaviour
  {
    [SerializeField] GameContext _currentGameContext = GameContext.Unspecified;
    protected List<Displayable> Objects { get; } = new();
    public List<Displayable> AllObjects => new(Objects);

    public GameContext GameContext
    {
      get => _currentGameContext == GameContext.Unspecified
        ? Errors.CheckEnum(DefaultGameContext())
        : _currentGameContext;

      set
      {
        Errors.CheckArgument(value != GameContext.Unspecified, "GameContext unspecified");
        _currentGameContext = value;
        StartCoroutine(MoveObjectsToPosition(true));
      }
    }

    public IEnumerator<YieldInstruction> AddObject(Displayable displayable, bool animate = true, int? index = null)
    {
      Insert(displayable, index);
      return MoveObjectsToPosition(animate);
    }

    public IEnumerator AddObjects(IEnumerable<Displayable> objects, bool animate = true)
    {
      foreach (var displayable in objects)
      {
        Insert(displayable, null);
      }

      return MoveObjectsToPosition(animate);
    }

    public int RemoveObject(Displayable displayable, bool animate = true)
    {
      var index = Objects.FindIndex(c => c == displayable);
      Errors.CheckNonNegative(index);
      Objects.RemoveAt(index);
      StartCoroutine(MoveObjectsToPosition(animate));
      return index;
    }

    public void RemoveObjectIfPresent(Displayable displayable, bool animate = true)
    {
      Objects.Remove(displayable);
      StartCoroutine(MoveObjectsToPosition(animate));
    }

    public void DebugUpdate()
    {
      StartCoroutine(MoveObjectsToPosition(true));
    }

    protected abstract GameContext DefaultGameContext();

    protected virtual float AnimationDuration => 0.3f;

    protected abstract Vector3 CalculateObjectPosition(int index, int count);

    protected virtual Vector3? CalculateObjectRotation(int index, int count) => null;

    void Insert(Displayable displayable, int? index)
    {
      displayable.Parent = this;
      if (!Objects.Contains(displayable))
      {
        if (index is { } i)
        {
          Objects.Insert(i, displayable);
        }
        else
        {
          Objects.Add(displayable);
        }
      }
    }

    IEnumerator<YieldInstruction> MoveObjectsToPosition(bool animate)
    {
      var sequence = DOTween.Sequence();
      for (var i = 0; i < Objects.Count; ++i)
      {
        var displayable = Objects[i];
        var position = CalculateObjectPosition(i, Objects.Count);
        var rotation = CalculateObjectRotation(i, Objects.Count);

        if (animate)
        {
          sequence.Insert(atPosition: 0, displayable.transform.DOMove(position, duration: AnimationDuration));
        }
        else
        {
          displayable.transform.position = position;
        }

        if (rotation is { } vector)
        {
          if (animate)
          {
            sequence.Insert(atPosition: 0, displayable.transform.DOLocalRotate(vector, duration: AnimationDuration));
          }
          else
          {
            displayable.transform.localEulerAngles = vector;
          }
        }

        displayable.SetGameContext(GameContext, 10 + i);
      }

      if (animate)
      {
        yield return sequence.WaitForCompletion();
      }
    }
  }
}