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

using System.Collections.Generic;
using UnityEngine;

namespace Spelldawn.Game
{
  public sealed class Arrow : MonoBehaviour
  {
    [SerializeField] float _pieceSize = 0.5f;
    [SerializeField] float _pieceFadeDistance = 0.35f;
    [SerializeField] GameObject _headPrefab = null!;
    [SerializeField] GameObject _piecePrefab = null!;
    Transform? _arrow;

    readonly List<Transform> _segments = new();
    readonly List<MeshRenderer> _renderers = new();

    public Vector3 Source { get; set; }
    public Vector3 Target { get; set; }

    void Update()
    {
      var distance = Vector3.Distance(Source, Target);
      var radius = (1f / 2f) + distance * distance / 8f;
      var diff = radius - 1f;
      var angle = 2f * Mathf.Acos(diff / radius);
      var length = angle * radius;
      var segmentAngle = _pieceSize / radius * Mathf.Rad2Deg;
      var center = new Vector3(0, -diff, distance / 2f);
      var left = Vector3.zero;
      var right = new Vector3(0, 0, distance);
      var segmentsCount = (int)(length / _pieceSize) + 1;

      while (_segments.Count < segmentsCount)
      {
        var segment = Instantiate(_piecePrefab, transform).transform;
        _segments.Add(segment);
        _renderers.Add(segment.GetComponent<MeshRenderer>());
      }

      for (var i = 0; i < _segments.Count; i++)
      {
        var segment = _segments[i].gameObject;
        if (segment.activeSelf != i < segmentsCount)
        {
          segment.SetActive(i < segmentsCount);
        }
      }

      var offset = Time.time * 1.5f * segmentAngle;
      var firstSegmentPos =
        Quaternion.Euler(Mathf.Repeat(offset, segmentAngle), 0f, 0f) * (left - center) + center;

      var fadeStartDistance = (Quaternion.Euler(segmentAngle / 2f, 0f, 0f) * (left - center) + center).z;

      for (var i = 0; i < segmentsCount; i++)
      {
        var pos = Quaternion.Euler(segmentAngle * i, 0f, 0f) * (firstSegmentPos - center) + center;
        _segments[i].localPosition = pos;
        _segments[i].localRotation = Quaternion.FromToRotation(Vector3.up, pos - center);

        var meshRenderer = _renderers[i];
        if (!meshRenderer)
        {
          continue;
        }

        var currentColor = meshRenderer.material.color;
        var distance1 = right.z - _pieceFadeDistance - pos.z;
        currentColor.a = Mathf.Clamp01(Mathf.Clamp01((pos.z - left.z) / fadeStartDistance) +
          Mathf.Clamp01(distance1 / fadeStartDistance) - 1f);
        meshRenderer.material.color = currentColor;
      }

      if (!_arrow)
      {
        _arrow = Instantiate(_headPrefab, transform).transform;
      }

      _arrow!.localPosition = right;
      _arrow.localRotation = Quaternion.FromToRotation(Vector3.up, right - center);
      transform.position = Source;
      transform.rotation = Quaternion.LookRotation(Target - Source, Vector3.up);
    }
  }
}