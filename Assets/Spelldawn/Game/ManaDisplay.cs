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
using Spelldawn.Services;
using Spelldawn.Utils;
using TMPro;
using UnityEngine;

#nullable enable

namespace Spelldawn.Game
{
  public sealed class ManaDisplay : MonoBehaviour
  {
    [SerializeField] Registry _registry = null!;
    [SerializeField] PlayerName _owner;
    [SerializeField] GameObject _pressEffect = null!;
    [SerializeField] GameObject _changeEffect = null!;
    [SerializeField] TextMeshPro _manaText = null!;
    [SerializeField] TextMeshPro _bonusManaText = null!;
    [SerializeField] TextMeshPro _manaSymbol = null!;
    [SerializeField] uint _currentMana = 5;
    [SerializeField] uint _currentBonusMana = 0;
    bool _animationDisabled;

    public uint CurrentMana => _currentMana;

    bool Clickable => _owner == PlayerName.User &&
                      _registry.CapabilityService.CanExecuteAction(GameAction.ActionOneofCase.GainMana);

    public void RenderManaDisplay(ManaView manaView)
    {
      SetMana(manaView.BaseMana);
      SetBonusMana(manaView.BonusMana);
    }

    public void DisableAnimation()
    {
      _manaSymbol.fontMaterial = new Material(Shader.Find("TextMeshPro/Distance Field"));
      _manaSymbol.color = new Color(0.15f, 0.78f, 0.85f);
      _animationDisabled = true;
    }

    public void GainMana(uint amount)
    {
      SetMana(_currentMana + amount);
    }

    public void SpendMana(uint amount)
    {
      Errors.CheckArgument(amount <= _currentMana, "Not enough mana available");
      SetMana(_currentMana - amount);
    }

    void SetMana(uint currentMana)
    {
      Errors.CheckNonNegative(currentMana);

      if (currentMana != _currentMana && !_animationDisabled)
      {
        _changeEffect.SetActive(false);
        _changeEffect.SetActive(true);
      }

      _currentMana = currentMana;
      _manaText.text = "" + _currentMana;
    }

    void SetBonusMana(uint bonusMana)
    {
      Errors.CheckNonNegative(bonusMana);

      if (bonusMana != _currentBonusMana)
      {
        _changeEffect.SetActive(false);
        _changeEffect.SetActive(true);
      }

      _currentBonusMana = bonusMana;
      
      _bonusManaText.gameObject.SetActive(bonusMana > 0);
      _bonusManaText.text = "" + _currentBonusMana;
    }    
    
    void OnMouseDown()
    {
      if (Clickable)
      {
        transform.localScale = 0.95f * Vector3.one;
      }
    }

    void OnMouseUp()
    {
      if (Clickable)
      {
        transform.localScale = Vector3.one;
      }
    }

    void OnMouseUpAsButton()
    {
      if (Clickable)
      {
        _pressEffect.SetActive(false);
        _pressEffect.SetActive(true);
        _registry.ActionService.HandleAction(new GameAction
        {
          GainMana = new GainManaAction()
        });
      }
    }
  }
}