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
        OnUpdated();
        _updateRequired = false;
      }
    }

    public IEnumerator AddObject(
      Displayable displayable,
      bool animate = true,
      bool animateRemove = true)
    {
      MarkUpdateRequired(animate);
      Insert(displayable, animateRemove);
      yield return new WaitUntil(() => !_animationRunning && !_updateRequired);
    }

    public IEnumerator AddObjects(
      IEnumerable<Displayable> objects,
      bool animate = true,
      bool animateRemove = true)
    {
      MarkUpdateRequired(animate);
      foreach (var displayable in objects)
      {
        Insert(displayable, animateRemove);
      }

      yield return new WaitUntil(() => !_animationRunning && !_updateRequired);
    }

    public uint RemoveObject(Displayable displayable, bool animate = true)
    {
      MarkUpdateRequired(animate);
      var index = _objects.FindIndex(c => c == displayable);
      Errors.CheckNonNegative(index);
      _objects.RemoveAt(index);
      displayable.Parent = null;
      return (uint)index;
    }

    public void RemoveObjectIfPresent(Displayable displayable, bool animate = true)
    {
      MarkUpdateRequired(animate);
      _objects.Remove(displayable);
      displayable.Parent = null;
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

    protected override void OnSetGameContext(GameContext oldContext, GameContext newContext, int? index = null)
    {
      MarkUpdateRequired(true);
    }

    public override void OnUpdateParentContainer()
    {
      MarkUpdateRequired(true);
    }

    protected abstract override GameContext DefaultGameContext();

    protected virtual float AnimationDuration => 0.3f;

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

    void Insert(Displayable displayable, bool animateRemove)
    {
      Errors.CheckNotNull(displayable);

      if (displayable.Parent)
      {
        displayable.Parent!.RemoveObjectIfPresent(displayable, animateRemove);
      }

      displayable.Parent = this;
      if (!_objects.Contains(displayable))
      {
        _objects.Add(displayable);
        _objects = _objects.OrderBy(o => o.SortingKey).ToList();
      }
    }

    void MoveObjectsToPosition(bool animate)
    {
      Sequence? sequence = null;
      if (animate)
      {
        _animationRunning = true;
        sequence = TweenUtils.Sequence($"{gameObject.name} MoveObjectsToPosition");
      }

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
          sequence.Insert(atPosition: 0, displayable.transform.DOMove(position, duration: AnimationDuration));
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
              displayable.transform.DOLocalRotate(vector, duration: AnimationDuration));
          }
          else
          {
            displayable.transform.localEulerAngles = vector;
          }
        }

        if (shouldAnimate)
        {
          sequence.Insert(atPosition: 0,
            displayable.transform.DOScale(Vector3.one * scale, duration: AnimationDuration));
        }
        else
        {
          displayable.transform.localScale = Vector3.one * scale;
        }

        displayable.SetGameContext(GameContext, 10 + i);
      }

      if (animate)
      {
        sequence.InsertCallback(AnimationDuration, () =>
        {
          _animationRunning = false;
          _animateNextUpdate = false;
        });
      }
    }
  }
}