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

using System.Collections.Generic;
using System.Linq;
using UnityEngine;

#nullable enable

namespace Spelldawn.Utils
{
  public static class CollectionUtils
  {
    public static IEnumerable<T> Interleave<T>(this IEnumerable<T> first, IEnumerable<T> second) =>
      first.Zip(second, (f, s) => new[] { f, s }).SelectMany(f => f);

    public static IEnumerator<YieldInstruction> Yield()
    {
      yield break;
    }

    public static IEnumerable<T> Once<T>(T value)
    {
      yield return value;
    }

    public static IEnumerable<T> WhereNotNull<T>(this IEnumerable<T?> source) where T : struct
    {
      return source.Where(t => t != null).Select(t => t.GetValueOrDefault());
    }

    public static IEnumerable<T> WhereNotNull<T>(this IEnumerable<T?> source) where T : class
    {
      return source.Where(t => t != null).Select(t => t!);
    }
  }
}