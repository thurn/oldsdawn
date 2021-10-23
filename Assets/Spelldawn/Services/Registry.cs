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

using Spelldawn.Game;
using Spelldawn.Protos;
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public sealed class Registry : MonoBehaviour
  {
    [SerializeField] Camera _mainCamera = null!;
    public Camera MainCamera => _mainCamera;

    [SerializeField] AssetService _assetService = null!;
    public AssetService AssetService => _assetService;

    [SerializeField] ObjectPoolService _objectPoolService = null!;
    public ObjectPoolService ObjectPoolService => _objectPoolService;

    [SerializeField] ActionService _actionService = null!;
    public ActionService ActionService => _actionService;

    [SerializeField] CardService _cardService = null!;
    public CardService CardService => _cardService;

    [SerializeField] CommandService _commandService = null!;
    public CommandService CommandService => _commandService;

    [SerializeField] DocumentService _documentService = null!;
    public DocumentService DocumentService => _documentService;

    public SampleData SampleData => _sampleData;
    [SerializeField] SampleData _sampleData = null!;

    [SerializeField] ArrowService _arrowService = null!;
    public ArrowService ArrowService => _arrowService;

    public ArenaService ArenaService => _arenaService;
    [SerializeField] ArenaService _arenaService = null!;

    [SerializeField] RaidService _raidService = null!;
    public RaidService RaidService => _raidService;

    [SerializeField] Hand _cardStaging = null!;
    public Hand CardStaging => _cardStaging;

    [SerializeField] Hand _userHand = null!;
    [SerializeField] Hand _opponentHand = null!;

    public Hand HandForPlayer(PlayerName playerName) => playerName == PlayerName.User ? _userHand : _opponentHand;

    [SerializeField] Transform _userDeckPosition = null!;
    [SerializeField] Transform _opponentDeckPosition = null!;

    public Transform DeckPositionForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userDeckPosition : _opponentDeckPosition;

    [SerializeField] Deck _userDeck = null!;
    [SerializeField] Deck _opponentDeck = null!;

    public Deck DeckForPlayer(PlayerName playerName) => playerName == PlayerName.User ? _userDeck : _opponentDeck;

    [SerializeField] Transform _cardStagingArea = null!;
    public Transform CardStagingArea => _cardStagingArea;

    [SerializeField] ManaDisplay _userManaDisplay = null!;
    [SerializeField] ManaDisplay _opponentManaDisplay = null!;

    public ManaDisplay ManaDisplayForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userManaDisplay : _opponentManaDisplay;

    [SerializeField] ActionDisplay _userActionDisplay = null!;
    [SerializeField] ActionDisplay _opponentActionDisplay = null!;

    public ActionDisplay ActionDisplayForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userActionDisplay : _opponentActionDisplay;

    [SerializeField] Transform _userIdentityCardPosition = null!;
    [SerializeField] Transform _opponentIdentityCardPosition = null!;

    public Transform IdentityCardPositionForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userIdentityCardPosition : _opponentIdentityCardPosition;

    [SerializeField] IdentityCard _userIdentityCard = null!;
    [SerializeField] IdentityCard _opponentIdentityCard = null!;

    public IdentityCard IdentityCardForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userIdentityCard : _opponentIdentityCard;

    void Awake()
    {
      Application.targetFrameRate = 60;
    }
  }
}