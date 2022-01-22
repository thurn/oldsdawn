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
using Spelldawn.Protos;
using Spelldawn.Services;
using UnityEngine;
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
      var previousChildrenCount = 0;
      VisualElement? result;
      var createdNewElement = false;

      if (previousElement != null && previousNode != null &&
          previousNode.NodeType?.NodeTypeCase == node.NodeType?.NodeTypeCase)
      {
        // If the previous node was of the same type as this node, mutate its VisualElement to match
        previousChildrenCount = previousNode.Children.Count;
        if (previousElement.childCount != previousChildrenCount)
        {
          throw new InvalidOperationException("Child count mismatch!");
        }

        for (var i = 0; i < previousChildrenCount; ++i)
        {
          if (i < node.Children.Count)
          {
            var child = node.Children[i];
            // Element exists in new tree, update it
            var updated = Update(
              registry,
              child,
              previousElement[i],
              i < previousNode.Children.Count ? previousNode.Children[i] : null);

            if (updated != null)
            {
              // New element was created for this position, replace existing element
              previousElement.Insert(i, updated);
              previousElement.RemoveAt(i + 1);
            }
          }
          else
          {
            // Element does not exist in new tree, delete it
            previousElement.RemoveAt(i);
          }
        }

        result = previousElement;
      }
      else
      {
        // Otherwise, create a new VisualElement matching this node
        result = Mason.CreateElement(node);
        createdNewElement = true;
      }

      for (var j = previousChildrenCount; j < node.Children.Count; ++j)
      {
        var child = node.Children[j];
        var updated = Update(registry, child);
        if (updated == null)
        {
          throw new InvalidOperationException($"Expected update for {child} to return a value");
        }
        else
        {
          result?.Add(updated);
        }
      }

      if (result != null)
      {
        Mason.ApplyToElement(registry, result, node);
      }

      return createdNewElement ? result : null;
    }
  }
}