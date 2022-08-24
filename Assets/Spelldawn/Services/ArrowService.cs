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
using Spelldawn.Game;
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class ArrowService : MonoBehaviour
  {
    public enum Type
    {
      Red,
      Green,
      Blue
    }

    public interface IArrowDelegate
    {
      void OnArrowMoved(Vector3 position);

      void OnArrowReleased(Vector3 position);
    }

    [SerializeField] Registry _registry = null!;
    [SerializeField] Arrow _redArrow = null!;
    [SerializeField] Arrow _greenArrow = null!;
    [SerializeField] Arrow _blueArrow = null!;

    [SerializeField] Arrow? _currentArrow;
    [SerializeField] Vector3 _startPosition;
    [SerializeField] float _dragStartScreenZ;
    IArrowDelegate? _delegate;

    public void ShowArrow(Type type, Transform start, IArrowDelegate arrowDelegate)
    {
      HideArrows();
      _currentArrow = ArrowForType(type);
      _startPosition = start.position;
      _delegate = arrowDelegate;
      _dragStartScreenZ = _registry.MainCamera.WorldToScreenPoint(start.position).z;
    }

    void Update()
    {
      if (_currentArrow && _currentArrow != null)
      {
        var mousePosition = _registry.MainCamera.ScreenToWorldPoint(
          new Vector3(Input.mousePosition.x, Input.mousePosition.y, _dragStartScreenZ));

        if (Input.GetMouseButton(0))
        {
          if (Vector3.Distance(_startPosition, mousePosition) < 3.0f)
          {
            _currentArrow.gameObject.SetActive(false);
          }
          else
          {
            _currentArrow.gameObject.SetActive(true);
            _currentArrow.Source = _startPosition;
            _currentArrow.Target = mousePosition;
            _delegate?.OnArrowMoved(mousePosition);
          }
        }
        else
        {
          _currentArrow.gameObject.SetActive(false);
          _currentArrow = null;
          _delegate?.OnArrowReleased(mousePosition);
        }
      }
    }

    public void HideArrows()
    {
      _currentArrow = null;
      _redArrow.gameObject.SetActive(false);
      _greenArrow.gameObject.SetActive(false);
      _blueArrow.gameObject.SetActive(false);
    }

    Arrow ArrowForType(Type type) => type switch
    {
      Type.Red => _redArrow,
      Type.Green => _greenArrow,
      Type.Blue => _blueArrow,
      _ => throw new ArgumentOutOfRangeException(nameof(type), type, null)
    };
  }
}