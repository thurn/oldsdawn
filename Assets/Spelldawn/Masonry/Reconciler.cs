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

using System.Collections.Generic;
using Spelldawn.Protos;
using Spelldawn.Services;
using Spelldawn.Utils;
using UnityEngine.UIElements;

#nullable enable

namespace Spelldawn.Masonry
{
  public static class Reconciler
  {
    /// <summary>
    /// Runs the tree diff algorithm, updating the Visual Element hierarchy to match the new node state.
    /// </summary>
    /// <para>
    /// This algorithm handles two cases: it generates a new VisualElement hierarchy from a Node, and it mutates
    /// a previously-generated VisualElement hierarchy to match a new Node.
    /// </para>
    /// <param name="registry">Service registry for asset fetching during rendering</param>
    /// <param name="node">The node to render</param>
    /// <param name="previousElement">Optionally, a previously-rendered VisualElement which should be updated to match
    /// the new Node state</param>
    /// <param name="previousNode">The Node value corresponding to <paramref name="previousElement"/></param>
    /// <returns>Either a new VisualElement matching the provided node, or null if <paramref name="previousElement"/>
    /// was mutated to match the provided node instead.</returns>
    public static VisualElement? Update(
      Registry registry,
      Node node,
      VisualElement? previousElement = null,
      Node? previousNode = null)
    {
      if (previousElement != null &&
          previousNode != null &&
          previousNode.NodeType?.NodeTypeCase == node.NodeType?.NodeTypeCase)
      {
        // If node types match, reuse this node
        return UpdateWhenMatching(registry, node, previousElement, previousNode);
      }
      else
      {
        return UpdateWhenNew(registry, node);
      }
    }

    static VisualElement? UpdateWhenMatching(
      Registry registry,
      Node node,
      VisualElement previousElement,
      Node previousNode)
    {
      var children = CreateChildren(registry, node, previousElement, previousNode);
      previousElement.Clear();
      foreach (var child in children)
      {
        previousElement.Add(child);
      }

      Mason.ApplyToElement(registry, previousElement, node);
      return null;
    }

    static VisualElement UpdateWhenNew(Registry registry, Node node)
    {
      // Otherwise, create a new VisualElement matching this node
      var result = Mason.CreateElement(node);
      foreach (var child in CreateChildren(registry, node))
      {
        result.Add(child);
      }

      Mason.ApplyToElement(registry, result, node);
      return result;
    }

    static List<VisualElement> CreateChildren(Registry registry,
      Node node,
      VisualElement? previousElement = null,
      Node? previousNode = null)
    {
      var children = new List<VisualElement>();
      for (var i = 0; i < node.Children.Count; ++i)
      {
        var child = node.Children[i];
        if (previousElement != null && previousNode != null && i < previousNode.Children.Count)
        {
          Errors.CheckState(previousElement.childCount == previousNode.Children.Count, "Child count mismatch");
          // Element exists in previous tree.
          var updated = Update(
            registry,
            child,
            previousElement[i],
            i < previousNode.Children.Count ? previousNode.Children[i] : null);
          children.Add(updated ?? previousElement[i]);
        }
        else
        {
          children.Add(UpdateWhenNew(registry, child));
        }
      }

      return children;
    }
  }
}