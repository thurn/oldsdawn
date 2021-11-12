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
using Spelldawn.Utils;
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class RaidService : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] ObjectDisplay _participants = null!;
    RoomId? _currentRoom;

    public ObjectDisplay RaidParticipants => _participants;

    public bool RaidActive => _currentRoom != null;

    public IEnumerator HandleInitiateRaid(InitiateRaidCommand command)
    {
      if (_currentRoom != command.RoomId)
      {
        _currentRoom = command.RoomId;
        _registry.BlackBackground.enabled = true;
        _registry.BlackBackground.color = new Color(0, 0, 0, 0.5f);
        _registry.ArenaService.LeftItems.SetGameContext(GameContext.ArenaRaidParticipant);

        var room = _registry.ArenaService.FindRoom(command.RoomId);
        switch (command.RoomId)
        {
          case RoomId.Sanctum:
            yield return _participants.AddObject(
              _registry.IdentityCardForPlayer(DataUtils.OpposingPlayer(command.Initiator)));
            break;
          case RoomId.Treasury:
            yield return _participants.AddObject(_registry.DeckForPlayer(DataUtils.OpposingPlayer(command.Initiator)));
            break;
          case RoomId.Crypts:
            yield return _participants.AddObject(
              _registry.DiscardPileForPlayer(DataUtils.OpposingPlayer(command.Initiator)));
            break;
        }

        yield return MoveToRaid(room.CardsInRoom.AllObjects);
        yield return MoveToRaid(room.Defenders.AllObjects);

        var identity = _registry.IdentityCardForPlayer(command.Initiator);
        identity.RaidSymbolShown = true;

        yield return _registry.ObjectPositionService.MoveGameObject(identity, new ObjectPosition
        {
          Raid = new ObjectPositionRaid()
        });
      }
    }

    IEnumerator MoveToRaid(IEnumerable<Displayable> cards)
    {
      var coroutines = new List<Coroutine>();
      foreach (var card in cards)
      {
        coroutines.Add(StartCoroutine(_registry.ObjectPositionService.MoveGameObject(card, new ObjectPosition
        {
          Raid = new ObjectPositionRaid()
        })));
      }

      foreach (var coroutine in coroutines)
      {
        yield return coroutine;
      }
    }

    public IEnumerator HandleEndRaid(EndRaidCommand endRaidCommand)
    {
      _registry.IdentityCardForPlayer(endRaidCommand.Initiator).RaidSymbolShown = false;
      _registry.BlackBackground.enabled = false;
      _currentRoom = null;
      yield break;
    }
  }
}