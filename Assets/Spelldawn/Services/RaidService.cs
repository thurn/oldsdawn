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
using System.Collections.Generic;
using Spelldawn.Game;
using Spelldawn.Protos;
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class RaidService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] SpriteRenderer _background = null!;
    [SerializeField] RaidDefendersCardDisplay _defenders = null!;
    [SerializeField] LinearCardDisplay _raidTargets = null!;
    RoomId? _currentRoom;

    public IEnumerator AddToRaid(AbstractCard card, CardPositionRaid position, bool animate) =>
      position.RoomLocation switch
      {
        RoomLocation.InRoom => _raidTargets.AddCard(card, animate, position.Index),
        _ => _defenders.AddCard(card, animate, position.Index),
      };

    public IEnumerator HandleInitiateRaid(InitiateRaidCommand command)
    {
      if (_currentRoom != command.RoomId)
      {
        if (_currentRoom is { } r && r != command.RoomId)
        {
          yield return HandleEndRaid(new EndRaidCommand());
        }

        _currentRoom = command.RoomId;
        _background.enabled = true;

        yield return MoveToRaid(_registry.ArenaService.FindRoom(command.RoomId).Defenders, RoomLocation.Defender);
        yield return MoveToRaid(_registry.ArenaService.FindRoom(command.RoomId).CardsInRoom, RoomLocation.InRoom);
      }
    }

    IEnumerator MoveToRaid(IEnumerable<AbstractCard> cards, RoomLocation roomLocation)
    {
      foreach (var card in cards)
      {
        yield return _registry.CardService.MoveCard(card, new CardPosition
        {
          Raid = new CardPositionRaid
          {
            RoomLocation = roomLocation
          }
        });
      }
    }

    public IEnumerator HandleEndRaid(EndRaidCommand endRaidCommand)
    {
      _background.enabled = false;

      if (_currentRoom != null)
      {
      }

      yield break;
    }
  }
}