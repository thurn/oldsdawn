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

using System;
using System.Collections.Generic;
using UnityEngine;
using UnityEngine.UIElements;

#nullable enable

namespace Spelldawn.Masonry
{
  public interface INodeCallbacks
  {
    VisualElement Self { get; }
    Lazy<Callbacks> Callbacks { get; }

    void SetCallback(Callbacks.Event eventType, Action? callback)
    {
      if (callback != null || Callbacks.IsValueCreated)
      {
        Callbacks.Value.SetCallback(Self, eventType, callback);
      }
    }
  }

  public sealed class Callbacks
  {
    public enum Event
    {
      Click,
      MouseDown,
      MouseUp,
      MouseEnter,
      MouseLeave
    }

    readonly HashSet<Event> _registered = new();
    readonly Dictionary<Event, Action?> _actions = new();
    float _lastClickTime;

    public void SetCallback(VisualElement e, Event eventType, Action? callback)
    {
      if (!_registered.Contains(eventType))
      {
        _registered.Add(eventType);
        Register(e, eventType);
      }

      _actions[eventType] = callback;
    }

    public bool HasCallback(Event eventType) => _registered.Contains(eventType);

    void Register(VisualElement e, Event eventType)
    {
      switch (eventType)
      {
        case Event.Click:
          e.RegisterCallback<ClickEvent>(OnClick);
          break;
        case Event.MouseDown:
          e.RegisterCallback<MouseDownEvent>(OnMouseDown);
          break;
        case Event.MouseUp:
          e.RegisterCallback<MouseUpEvent>(OnMouseUp);
          break;
        case Event.MouseEnter:
          e.RegisterCallback<MouseEnterEvent>(OnMouseEnter);
          break;
        case Event.MouseLeave:
          e.RegisterCallback<MouseLeaveEvent>(OnMouseLeave);
          break;
        default:
          throw new ArgumentOutOfRangeException(nameof(eventType), eventType, "Unknown event type");
      }
    }

    public void OnClick(ClickEvent evt)
    {
      if (Mathf.Abs(Time.time - _lastClickTime) > 0.1f)
      {
        _actions[Event.Click]?.Invoke();
        _lastClickTime = Time.time;
      }
    }

    void OnMouseDown(MouseDownEvent evt)
    {
      _actions[Event.MouseDown]?.Invoke();
    }

    void OnMouseUp(MouseUpEvent evt)
    {
      _actions[Event.MouseUp]?.Invoke();
    }

    void OnMouseEnter(MouseEnterEvent evt)
    {
      _actions[Event.MouseEnter]?.Invoke();
    }

    void OnMouseLeave(MouseLeaveEvent evt)
    {
      _actions[Event.MouseLeave]?.Invoke();
    }
  }

  public sealed class NodeVisualElement : VisualElement, INodeCallbacks
  {
    public VisualElement Self => this;
    readonly Lazy<Callbacks> _callbacks = new();
    public Lazy<Callbacks> Callbacks => _callbacks;
  }

  public sealed class NodeLabel : Label, INodeCallbacks
  {
    public VisualElement Self => this;
    readonly Lazy<Callbacks> _callbacks = new();
    public Lazy<Callbacks> Callbacks => _callbacks;
  }
  
  public sealed class NodeImage : Image, INodeCallbacks
  {
    public VisualElement Self => this;
    readonly Lazy<Callbacks> _callbacks = new();
    public Lazy<Callbacks> Callbacks => _callbacks;
  }  
}