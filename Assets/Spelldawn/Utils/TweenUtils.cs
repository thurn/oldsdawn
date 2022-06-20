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

using DG.Tweening;

#nullable enable

namespace Spelldawn.Utils
{
  public static class TweenUtils
  {
    public const float GlobalAnimationMultiplier = 1.0f;
    public const float MoveAnimationDurationSeconds = 0.3f * GlobalAnimationMultiplier;
    public const float FlipAnimationDurationSeconds = 0.4f * GlobalAnimationMultiplier;

    public static Sequence Sequence(string name)
    {
      var result = DOTween.Sequence();
      result.stringId = name;
      return result;
    }
  }
}