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

using Spelldawn.Protos;

#nullable enable

namespace Spelldawn.Utils
{
  public static class IdUtil
  {
    public static readonly GameObjectId UserCardId = new()
    {
      CardId = new CardId
      {
        IdentityCard = PlayerName.User
      }
    };

    public static readonly GameObjectId OpponentCardId = new()
    {
      CardId = new CardId
      {
        IdentityCard = PlayerName.Opponent
      }
    };

    public static GameObjectId CardObjectId(CardView card) => CardObjectId(card.CardId);

    public static GameObjectId CardObjectId(CardId cardId) => new()
    {
      CardId = cardId
    };
  }
}