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

#if UNITY_EDITOR && UNITY_IOS

using System.IO;
using UnityEngine;
using UnityEditor;
using UnityEditor.Callbacks;
using UnityEditor.iOS.Xcode;

public class BuildIos
{
  [PostProcessBuild]
  public static void OnPostProcessBuild(BuildTarget target, string path)
  {
    Debug.Log("Post-processing iOS Build");
    var projectPath = PBXProject.GetPBXProjectPath(path);
    var project = new PBXProject();
    project.ReadFromString(File.ReadAllText(projectPath));
#if UNITY_2019_3_OR_NEWER
    var targetGuid = project.GetUnityFrameworkTargetGuid();
#else
    var targetGuid = project.TargetGuidByName(PBXProject.GetUnityTargetName());
#endif

    // libz.tbd for grpc ios build
    project.AddFrameworkToProject(targetGuid, "libz.tbd", false);

    // bitode is disabled for libgrpc_csharp_ext, so need to disable it for the whole project
    project.SetBuildProperty(targetGuid, "ENABLE_BITCODE", "NO");

    File.WriteAllText(projectPath, project.WriteToString());
  }
}

#endif