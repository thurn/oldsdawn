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
using System.Linq;
using DG.Tweening;
using Spelldawn.Services;
using Spelldawn.Utils;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public abstract class ObjectDisplay : Displayable
  {
    [Space(10)] [Header("Object Display")] [SerializeField]
    List<Displayable> _objects = new();

    [SerializeField] bool _updateRequired;

    [SerializeField] bool _animateNextUpdate;

    [SerializeField] bool _animationRunning;

    public List<Displayable> AllObjects => new(_objects);

    protected int ObjectCount => _objects.Count;

    protected abstract Registry? Registry { get; }

    protected override void OnStart()
    {
      _updateRequired = true;
      foreach (var child in _objects)
      {
        child.Parent = this;
      }
    }

    void Update()
    {
      if (_updateRequired && !_animationRunning)
      {
        MoveObjectsToPosition(_animateNextUpdate);
        _updateRequired = false;
      }

      OnUpdated();
    }

    public IEnumerator AddObject(Displayable displayable, bool animate = true)
    {
      if (animate)
      {
        Insert(displayable, animate: true);
        MarkUpdateRequired(animate: true);
        yield return WaitUntilIdle();
      }
      else
      {
        AddObjectImmediate(displayable);
      }
    }

    /// <summary>Insert a Displayable into this container immediately, with no animation.</summary>
    public void AddObjectImmediate(Displayable displayable)
    {
      Insert(displayable, animate: false);
      MoveObjectsToPosition(animate: false);
      OnUpdated();
    }

    public IEnumerator AddObjects(List<Displayable> objects)
    {
      foreach (var displayable in objects)
      {
        Insert(displayable, animate: true);
      }
      
      MarkUpdateRequired(animate: true);
      yield return WaitUntilIdle();
    }

    public void RemoveObject(Displayable displayable, bool animate = true)
    {
      var index = _objects.FindIndex(c => c == displayable);
      Errors.CheckNonNegative(index);
      _objects.RemoveAt(index);
      displayable.Parent = null;
      MarkUpdateRequired(animate);
    }

    /// <summary>Tries to remove an object from this ObjectDisplay, returning true if the object was removed.</summary>
    public void RemoveObjectIfPresent(Displayable displayable, bool animate = true)
    {
      if (_objects.Contains(displayable))
      {
        RemoveObject(displayable, animate);
      }
    }

    public void DestroyAll()
    {
      foreach (var displayable in _objects)
      {
        Destroy(displayable.gameObject);
      }

      _objects.Clear();
    }

    public void DebugUpdate()
    {
      if (Application.isPlaying)
      {
        MarkUpdateRequired(true);
      }
      else
      {
        MoveObjectsToPosition(false);
      }
    }

    public override bool IsContainer() => true;

    public WaitUntil WaitUntilIdle() => new(() => !_animationRunning && !_updateRequired);

    protected override void OnSetGameContext(GameContext oldContext, GameContext newContext)
    {
      MarkUpdateRequired(true);
    }

    public override void OnUpdateParentContainer()
    {
      MarkUpdateRequired(true);
    }

    protected abstract override GameContext DefaultGameContext();

    protected abstract Vector3 CalculateObjectPosition(int index, int count);

    protected virtual Vector3? CalculateObjectRotation(int index, int count) => null;

    protected virtual float? CalculateObjectScale(int index, int count) => null;

    protected virtual void OnUpdated()
    {
    }

    void MarkUpdateRequired(bool animate)
    {
      _updateRequired = true;
      _animateNextUpdate |= animate;
    }

    void Insert(Displayable displayable, bool animate = true)
    {
      Errors.CheckNotNull(displayable);

      if (!_objects.Contains(displayable))
      {
        if (displayable.Parent)
        {
          displayable.Parent!.RemoveObjectIfPresent(displayable, animate);
        }

        displayable.Parent = this;
        _objects.Add(displayable);
      }

      // Even if the object is already present, the sorting order of elements might have changed.
      _objects = _objects.OrderBy(o => o.SortingKey).ThenBy(o => o.SortingSubkey).ToList();
    }

    void MoveObjectsToPosition(bool animate)
    {
      Sequence? sequence = null;
      if (animate)
      {
        _animationRunning = true;
        sequence = TweenUtils.Sequence($"{gameObject.name} MoveObjectsToPosition").SetEase(Ease.InOutSine);
      }

      const float duration = TweenUtils.MoveAnimationDurationSeconds;

      for (var i = 0; i < _objects.Count; ++i)
      {
        var displayable = _objects[i];
        var position = CalculateObjectPosition(i, _objects.Count);
        var rotation = CalculateObjectRotation(i, _objects.Count);
        var scale = CalculateObjectScale(i, _objects.Count) ?? displayable.DefaultScale;

        var shouldAnimate = animate;
        if (displayable.IsContainer())
        {
          // If the object is itself a container, we jump it to the destination position and then
          // schedule its internal animations.
          shouldAnimate = false;
          displayable.OnUpdateParentContainer();
        }

        if (shouldAnimate)
        {
          sequence.Insert(atPosition: 0, displayable.transform.DOMove(position, duration));
        }
        else
        {
          displayable.transform.position = position;
        }

        if (rotation is { } vector)
        {
          if (shouldAnimate)
          {
            sequence.Insert(atPosition: 0,
              displayable.transform.DOLocalRotate(vector, duration));
          }
          else
          {
            displayable.transform.localEulerAngles = vector;
          }
        }

        if (shouldAnimate)
        {
          sequence.Insert(atPosition: 0,
            displayable.transform.DOScale(Vector3.one * scale, duration));
        }
        else
        {
          displayable.transform.localScale = Vector3.one * scale;
        }

        displayable.SetGameContext(GameContext);
      }

      if (animate)
      {
        sequence.InsertCallback(duration, () =>
        {
          _animationRunning = false;
          _animateNextUpdate = false;
        });
      }
    }
  }
}