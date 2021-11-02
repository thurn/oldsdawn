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
using System.Collections;
using DG.Tweening;
using Spelldawn.Protos;
using Spelldawn.Utils;
using TMPro;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class GameMessage : MonoBehaviour
  {
    [Serializable]
    public class MessageContent
    {
      [SerializeField] TimedEffect _effect = null!;
      public TimedEffect Effect => _effect;

      [SerializeField] TextMeshPro _text = null!;
      public TextMeshPro Text => _text;
    }

    [SerializeField] MessageContent _dawn = null!;
    [SerializeField] MessageContent _dusk = null!;
    [SerializeField] MessageContent _victory = null!;
    [SerializeField] MessageContent _defeat = null!;

    public IEnumerator Show(DisplayGameMessageCommand command) => command.MessageType switch
    {
      GameMessageType.Dawn => ShowContent(_dawn, 1.75f),
      GameMessageType.Dusk => ShowContent(_dusk, 1.75f),
      GameMessageType.Victory => ShowContent(_victory, 1.5f),
      GameMessageType.Defeat => ShowContent(_defeat, 1.5f),
      _ => CollectionUtils.Yield()
    };

    IEnumerator ShowContent(MessageContent content, float durationSeconds)
    {
      content.Effect.gameObject.SetActive(false);
      content.Effect.gameObject.SetActive(true);
      content.Text.gameObject.SetActive(true);
      content.Text.alpha = 0f;
      yield return DOTween
        .To(() => content.Text.alpha, x => content.Text.alpha = x, endValue: 1f, 0.2f)
        .WaitForCompletion();
      yield return new WaitForSeconds(durationSeconds);
      yield return DOTween
        .To(() => content.Text.alpha, x => content.Text.alpha = x, endValue: 0f, 0.2f)
        .WaitForCompletion();
      content.Text.gameObject.SetActive(false);
    }
  }
}