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

using TMPro;
using UnityEngine;

namespace Spelldawn.Game
{
  public sealed class ActionSymbol : MonoBehaviour
  {
    [SerializeField] TextMeshPro _text = null!;
    [SerializeField] GameObject _rotationAxis = null!;
    [SerializeField] float _speed = 300f;
    [SerializeField] Material _activeMaterial = null!;
    [SerializeField] Material _inactiveMaterial = null!;
    
    bool _filled = true;
    bool _animate;
    float _total;
    Vector3 _startPosition;
    Vector3? _rotatedPosition;
    Material? _setMaterialTo;

    public bool IsAnimating => _animate;

    void Start()
    {
      _startPosition = transform.position;
    }

    public void SetFilled(bool filled)
    {
      if (_animate)
      {
        // Animation already running
        return;
      }

      if (filled)
      {
        gameObject.SetActive(true);
      }
      
      switch (filled)
      {
        case true when !_filled:
          _setMaterialTo = _activeMaterial;
          _animate = true;
          break;
        case false when _filled:
          _setMaterialTo = _inactiveMaterial;
          _animate = true;
          break;
      }

      _filled = filled;
    }

    public void SetFontMaterial(Material material)
    {
      _text.fontMaterial = material;
      _activeMaterial = material;
    }

    void Update()
    {
      // The world's most sketchy rotate animation. I couldn't get the obvious ways of
      // doing this to work because I am bad at Unity. As Blaise Pascal once said,
      // "If I had more time, I would have written you some shorter code".
      
      if (_animate)
      {
        var degrees = _speed * Time.deltaTime;
        var axis = new Vector3(0, -1f, -0.2f); // MainCamera.forward
        transform.RotateAround(_rotationAxis.transform.position, axis, degrees);
        _total += degrees;

        if (_total >= 150f && _setMaterialTo)
        {
          _text.fontMaterial = _setMaterialTo;
          _setMaterialTo = null;
        }

        if (_total >= 180.0f)
        {
          _animate = false;
          _total = 0;

          if (Mathf.Abs(transform.localEulerAngles.z) < 1f)
          {
            // The position and angles get slightly offset after several rotations, causing the object
            // to move slightly, so we save the first positions we see and snap it back to those.
            transform.localEulerAngles = Vector3.zero;
            transform.position = _startPosition;
          }
          else
          {
            transform.localEulerAngles = new Vector3(0f, 0f, 180f);
            _rotatedPosition ??= transform.position;
            transform.position = _rotatedPosition.Value;
          }
        }
      }
    }
  }
}