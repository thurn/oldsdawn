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

using UnityEngine;
using System.Collections;
using TMPro;

namespace Spelldawn.Game
{
  /// <summary>Curves text along a predefined curve, typically used for card titles.</summary>
  public sealed class WarpText : MonoBehaviour
  {
    const float Tangent = 5f;

    static readonly AnimationCurve Curve = new(
      new Keyframe(0f, 0f)
      {
        outTangent = Tangent
      },
      new Keyframe(0.5f, 1.0f),
      new Keyframe(1.0f, 0f)
      {
        inTangent = -Tangent
      })
    {
      preWrapMode = WrapMode.Clamp,
      postWrapMode = WrapMode.Clamp
    };

    [SerializeField] TMP_Text _text = null!;

    void Start()
    {
      StartCoroutine(Warp());
    }

    public IEnumerator Warp()
    {
      for (var i = 0; i < 10; ++i)
      {
        // TextMeshPro mesh updates basically just take an arbitrary number of frames to take effect,
        // and I don't think there's really a callback when it's done.
        //
        // The *official* Unity example of how to do this is just "run it in a loop forever lol" ?!?
        yield return new WaitForEndOfFrame();
        RunWarp();
      }      
    }

    /// <summary>
    ///  Method to curve text along a Unity animation curve.
    /// </summary>
    void RunWarp()
    {
      var preferredWidth = _text.GetPreferredValues().x;

      if (preferredWidth < 16f)
      {
        // Text is not wide enough to curve
        _text.transform.localPosition = new Vector3(0, 1.94f, 0);
        return;
      }

      float curveScale;
      if (preferredWidth < 19f)
      {
        curveScale = 0.5f;
        _text.transform.localPosition = new Vector3(0, 1.90f, 0);
      }
      else
      {
        curveScale = 0.75f;
        _text.transform.localPosition = new Vector3(0, 1.86f, 0);
      }
      
      // Text warping code below based on Unity's 'WarpTextExample.cs'.

      _text.havePropertiesChanged = true; // Need to force the TextMeshPro Object to be updated.
      _text.ForceMeshUpdate();
      var textInfo = _text.textInfo;
      var characterCount = textInfo.characterCount;
      var boundsMinX = _text.bounds.min.x;
      var boundsMaxX = _text.bounds.max.x;

      for (var i = 0; i < characterCount; ++i)
      {
        if (!textInfo.characterInfo[i].isVisible)
        {
          continue;
        }

        var vertexIndex = textInfo.characterInfo[i].vertexIndex;
        
        // Get the index of the mesh used by this character.
        var materialIndex = textInfo.characterInfo[i].materialReferenceIndex;
        var vertices = textInfo.meshInfo[materialIndex].vertices;
        
        // Compute the baseline mid point for each character
        var offsetToMidBaseline = new Vector3(
          (vertices[vertexIndex + 0].x + vertices[vertexIndex + 2].x) / 2,
          textInfo.characterInfo[i].baseLine,
          0f);

        // Apply offset to adjust our pivot point.
        vertices[vertexIndex + 0] += -offsetToMidBaseline;
        vertices[vertexIndex + 1] += -offsetToMidBaseline;
        vertices[vertexIndex + 2] += -offsetToMidBaseline;
        vertices[vertexIndex + 3] += -offsetToMidBaseline;

        // Compute the angle of rotation for each character based on the animation curve
        var x0 = (offsetToMidBaseline.x - boundsMinX) /
                 (boundsMaxX - boundsMinX); // Character's position relative to the bounds of the mesh.
        
        var x1 = x0 + 0.0001f;
        var y0 = Curve.Evaluate(x0) * curveScale;
        var y1 = Curve.Evaluate(x1) * curveScale;

        var horizontal = new Vector3(1, 0, 0);
        var tangent = new Vector3(x1 * (boundsMaxX - boundsMinX) + boundsMinX, y1) -
                      new Vector3(offsetToMidBaseline.x, y0);

        var dot = Mathf.Acos(Vector3.Dot(horizontal, tangent.normalized)) * 57.2957795f;
        var cross = Vector3.Cross(horizontal, tangent);
        var angle = cross.z > 0 ? dot : 360 - dot;

        var matrix = Matrix4x4.TRS(new Vector3(0, y0, 0), Quaternion.Euler(0, 0, angle), Vector3.one);

        vertices[vertexIndex + 0] = matrix.MultiplyPoint3x4(vertices[vertexIndex + 0]);
        vertices[vertexIndex + 1] = matrix.MultiplyPoint3x4(vertices[vertexIndex + 1]);
        vertices[vertexIndex + 2] = matrix.MultiplyPoint3x4(vertices[vertexIndex + 2]);
        vertices[vertexIndex + 3] = matrix.MultiplyPoint3x4(vertices[vertexIndex + 3]);

        vertices[vertexIndex + 0] += offsetToMidBaseline;
        vertices[vertexIndex + 1] += offsetToMidBaseline;
        vertices[vertexIndex + 2] += offsetToMidBaseline;
        vertices[vertexIndex + 3] += offsetToMidBaseline;
      }

      // Upload the mesh with the revised information
      _text.UpdateVertexData();
    }
  }
}