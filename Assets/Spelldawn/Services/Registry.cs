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

using System.Linq;
using Spelldawn.Game;
using Spelldawn.Protos;
using Spelldawn.Tests;
using UnityEngine;

#nullable enable

namespace Spelldawn.Services
{
  public enum GlobalGameMode
  {
    Default,
    ScreenshotTest
  }

  public sealed class Registry : MonoBehaviour
  {
    [SerializeField] GlobalGameMode _globalGameMode;
    public GlobalGameMode GlobalGameMode => _globalGameMode;

    [SerializeField] Camera _mainCamera = null!;
    public Camera MainCamera => _mainCamera;

    [SerializeField] Reporter _logViewer = null!;
    public Reporter LogViewer => _logViewer;

    [SerializeField] AudioSource _mainAudioSource = null!;
    public AudioSource MainAudioSource => _mainAudioSource;

    [SerializeField] GameService _gameService = null!;
    public GameService GameService => _gameService;

    [SerializeField] AssetService _assetService = null!;
    public AssetService AssetService => _assetService;

    [SerializeField] AssetPoolService _assetPoolService = null!;
    public AssetPoolService AssetPoolService => _assetPoolService;

    [SerializeField] ActionService _actionService = null!;
    public ActionService ActionService => _actionService;

    [SerializeField] ObjectPositionService _objectPositionService = null!;
    public ObjectPositionService ObjectPositionService => _objectPositionService;

    [SerializeField] CardService _cardService = null!;
    public CardService CardService => _cardService;

    [SerializeField] CommandService _commandService = null!;
    public CommandService CommandService => _commandService;

    [SerializeField] DocumentService _documentService = null!;
    public DocumentService DocumentService => _documentService;

    [SerializeField] MusicService _musicService = null!;
    public MusicService MusicService => _musicService;

    [SerializeField] ArrowService _arrowService = null!;
    public ArrowService ArrowService => _arrowService;

    public ArenaService ArenaService => _arenaService;
    [SerializeField] ArenaService _arenaService = null!;

    [SerializeField] RaidService _raidService = null!;
    public RaidService RaidService => _raidService;

    [SerializeField] StackObjectDisplay _offscreenCards = null!;
    public StackObjectDisplay OffscreenCards => _offscreenCards;

    [SerializeField] CurveObjectDisplay _cardStaging = null!;
    public CurveObjectDisplay CardStaging => _cardStaging;
    
    [SerializeField] CurveObjectDisplay _revealedCardsBrowserSmall = null!;
    public CurveObjectDisplay RevealedCardsBrowserSmall => _revealedCardsBrowserSmall;
    
    [SerializeField] CurveObjectDisplay _revealedCardsBrowserLarge = null!;
    public CurveObjectDisplay RevealedCardsBrowserLarge => _revealedCardsBrowserLarge;    

    [SerializeField] CardBrowser _cardBrowser = null!;
    public CardBrowser CardBrowser => _cardBrowser;

    [SerializeField] StackObjectDisplay _cardScoring = null!;
    public StackObjectDisplay CardScoring => _cardScoring;

    [SerializeField] GameMessage _gameMessage = null!;
    public GameMessage GameMessage => _gameMessage;

    [SerializeField] BackgroundOverlay _backgroundOverlay = null!;
    public BackgroundOverlay BackgroundOverlay => _backgroundOverlay;

    [SerializeField] StaticAssets _staticAssets = null!;
    public StaticAssets StaticAssets => _staticAssets;

    [SerializeField] RewardChest _rewardChest = null!;
    public RewardChest RewardChest => _rewardChest;

    [SerializeField] CurveObjectDisplay _userHand = null!;
    [SerializeField] CurveObjectDisplay _opponentHand = null!;

    public CurveObjectDisplay HandForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userHand : _opponentHand;

    [SerializeField] ObjectDisplay _userDeckPosition = null!;
    [SerializeField] ObjectDisplay _opponentDeckPosition = null!;

    public ObjectDisplay DeckPositionForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userDeckPosition : _opponentDeckPosition;

    [SerializeField] Deck _userDeck = null!;
    [SerializeField] Deck _opponentDeck = null!;

    public Deck DeckForPlayer(PlayerName playerName) => playerName == PlayerName.User ? _userDeck : _opponentDeck;

    [SerializeField] ObjectDisplay _userDiscardPilePosition = null!;
    [SerializeField] ObjectDisplay _opponentDiscardPilePosition = null!;

    public ObjectDisplay DiscardPilePositionForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userDiscardPilePosition : _opponentDiscardPilePosition;

    [SerializeField] DiscardPile _userDiscardPile = null!;
    [SerializeField] DiscardPile _opponentDiscardPile = null!;

    public DiscardPile DiscardPileForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userDiscardPile : _opponentDiscardPile;

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

    [SerializeField] ObjectDisplay _userIdentityCardPosition = null!;
    [SerializeField] ObjectDisplay _opponentIdentityCardPosition = null!;

    public ObjectDisplay IdentityCardPositionForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userIdentityCardPosition : _opponentIdentityCardPosition;

    [SerializeField] IdentityCard _userIdentityCard = null!;
    [SerializeField] IdentityCard _opponentIdentityCard = null!;

    public IdentityCard IdentityCardForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userIdentityCard : _opponentIdentityCard;

    [SerializeField] GameObject _userActiveLight = null!;
    [SerializeField] GameObject _opponentActiveLight = null!;

    public GameObject ActiveLightForPlayer(PlayerName playerName) =>
      playerName == PlayerName.User ? _userActiveLight : _opponentActiveLight;

    [SerializeField] GameObject _graphy = null!;
    public GameObject Graphy => _graphy;

    public ScreenshotTestService? ScreenshotTests { get; private set; }

    void Start()
    {
      Application.targetFrameRate = 60;
      var runTests = false;

      if (_globalGameMode == GlobalGameMode.ScreenshotTest ||
          System.Environment.GetCommandLineArgs().Any(arg => arg.Contains("test")))
      {
        _globalGameMode = GlobalGameMode.ScreenshotTest;
        ScreenshotTests = ScreenshotTestService.Initialize(this, out runTests);
      }

      ActionService.Initialize();
      DocumentService.Initialize();
      GameService.Initialize(_globalGameMode);
      MusicService.Initialize(_globalGameMode);

      if (runTests)
      {
        ScreenshotTests!.RunTests();
      }
    }
  }
}