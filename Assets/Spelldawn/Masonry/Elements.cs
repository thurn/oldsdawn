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
using UnityEngine.UIElements;

#nullable enable

namespace Spelldawn.Masonry
{
  public interface INodeCallbacks
  {
    public void SetCallback<TEventType>(
      EventCallback<TEventType>? callback,
      TrickleDown useTrickleDown = TrickleDown.NoTrickleDown) where TEventType : EventBase<TEventType>, new();
  }

  public sealed class Callbacks
  {
    readonly Dictionary<Type, object> _callbacks = new();

    public void SetCallback<TEventType>(
      VisualElement e,
      EventCallback<TEventType>? callback,
      TrickleDown useTrickleDown = TrickleDown.NoTrickleDown)
      where TEventType : EventBase<TEventType>, new()
    {
      var t = typeof(TEventType);
      if (_callbacks.ContainsKey(t))
      {
        e.UnregisterCallback((EventCallback<TEventType>)_callbacks[t]);
        _callbacks.Remove(t);
      }

      if (callback != null)
      {
        e.RegisterCallback(callback, useTrickleDown);
        _callbacks[t] = callback;
      }
    }
  }

  public sealed class NodeVisualElement : VisualElement, INodeCallbacks
  {
    readonly Lazy<Callbacks> _callbacks = new();

    public void SetCallback<TEventType>(
      EventCallback<TEventType>? callback,
      TrickleDown useTrickleDown = TrickleDown.NoTrickleDown) where TEventType : EventBase<TEventType>, new()
    {
      if (callback != null || _callbacks.IsValueCreated)
      {
        _callbacks.Value.SetCallback(this, callback, useTrickleDown);
      }
    }
  }

  public sealed class NodeLabel : Label, INodeCallbacks
  {
    readonly Lazy<Callbacks> _callbacks = new();

    public void SetCallback<TEventType>(
      EventCallback<TEventType>? callback,
      TrickleDown useTrickleDown = TrickleDown.NoTrickleDown) where TEventType : EventBase<TEventType>, new()
    {
      if (callback != null || _callbacks.IsValueCreated)
      {
        _callbacks.Value.SetCallback(this, callback, useTrickleDown);
      }
    }
  }
}