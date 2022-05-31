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

#nullable enable

using System.Runtime.InteropServices;
using System.Text;
using Google.Protobuf;
using Spelldawn.Protos;
using UnityEngine;

namespace Spelldawn.Utils
{
  static class Plugin
  {
    const int BufferSize = 32_768;

    public static void Initialize()
    {
      var path = $"{Application.persistentDataPath}/db";
      Debug.Log($"Setting database path to {path}");
      var encoded = Encoding.UTF8.GetBytes(path);
      Errors.CheckNonNegative(spelldawn_initialize(encoded, encoded.Length));
    }

    public static CommandList? Connect(ConnectRequest request)
    {
      var input = request.ToByteArray();
      var output = new byte[BufferSize];
      var responseSize = spelldawn_connect(input, input.Length, output, output.Length);
      return responseSize > 0 ? CommandList.Parser.ParseFrom(output, 0, responseSize) : null;
    }

    public static CommandList PerformAction(GameRequest request)
    {
      var input = request.ToByteArray();
      var output = new byte[BufferSize];
      var responseSize = Errors.CheckNonNegative(spelldawn_perform_action(input, input.Length, output, output.Length));
      return CommandList.Parser.ParseFrom(output, 0, responseSize);
    }

#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
      [DllImport("__Internal")]
#else
    [DllImport("libspelldawn")]
#endif
    public static extern int spelldawn_initialize(byte[] path, int pathLength);

#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
    [DllImport("__Internal")]
#else
    [DllImport("libspelldawn")]
#endif
    public static extern int spelldawn_connect(
      byte[] request,
      int requestLength,
      [Out] byte[] response,
      int responseLength);

#if !UNITY_EDITOR && (UNITY_IOS || UNITY_WEBGL)
    [DllImport("__Internal")]
#else
    [DllImport("libspelldawn")]
#endif
    public static extern int spelldawn_perform_action(
      byte[] request,
      int requestLength,
      [Out] byte[] response,
      int responseLength);
  }
}