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
using DG.Tweening;
using Spelldawn.Protos;
using Spelldawn.Services;
using Spelldawn.Utils;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  [DisallowMultipleComponent]
  public sealed class Projectile : MonoBehaviour
  {
    [SerializeField] float _scale = 3f;
    [SerializeField] TimedEffect? _flash;
    [SerializeField] TimedEffect? _hit;

    public void EditorSetEffects(TimedEffect? hit, TimedEffect? flash)
    {
      _hit = hit;
      _flash = flash;
    }

    public IEnumerator Fire(
      Registry registry,
      Transform target,
      TimeValue? duration,
      EffectAddress? additionalHit,
      TimeValue? additionalHitDelay)
    {
      transform.localScale = _scale * Vector3.one;
      transform.LookAt(target);
      var rotation = Quaternion.LookRotation(transform.position - target.position);

      if (_flash && _flash != null)
      {
        var flash = registry.ObjectPoolService.Create(_flash, transform.position);
        flash.transform.rotation = rotation;
        flash.transform.localScale = _scale * Vector3.one;
      }

      yield return DOTween.Sequence()
        .Append(transform.DOMove(target.position, DataUtils.ToSeconds(duration, 300)).SetEase(Ease.Linear))
        .WaitForCompletion();

      TimedEffect? hit = null;
      if (_hit && _hit != null)
      {
        hit = registry.ObjectPoolService.Create(_hit, transform.position);
        hit.transform.rotation = rotation;
        hit.transform.localScale = _scale * Vector3.one;
      }

      gameObject.SetActive(value: false);

      if (additionalHit != null)
      {
        yield return new WaitForSeconds(DataUtils.ToSeconds(additionalHitDelay, 0));
        var additionalHitEffect =
          registry.ObjectPoolService.Create(registry.AssetService.GetEffect(additionalHit), transform.position);
        additionalHitEffect.transform.rotation = rotation;

        if (hit)
        {
          hit!.gameObject.SetActive(false);
        }
      }
    }

    void OnValidate()
    {
      foreach (var ps in GetComponentsInChildren<ParticleSystem>())
      {
        SortingOrder.Create(GameContext.Effects).ApplyTo(ps.GetComponent<Renderer>());
      }
    }
  }
}