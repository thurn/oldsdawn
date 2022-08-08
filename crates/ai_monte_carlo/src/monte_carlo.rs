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

//! An implementation of Monte Carlo Tree Search.
//!
//! This implementation is based on the pseudocode given in "A Survey of Monte
//! Carlo Tree Search Methods" by Browne et al. in IEEE Transactions on
//! Computational Intelligence and AI in Games, Vol. 4, No. 1, March 2012.

use std::collections::HashSet;
use std::time::Instant;

use ai_core::game_state_node::{GameStateNode, GameStatus};
use ai_core::selection_algorithm::SelectionAlgorithm;
use ai_core::state_evaluator::StateEvaluator;
use anyhow::Result;
use petgraph::prelude::{EdgeRef, NodeIndex};
use petgraph::{Direction, Graph};
use rand::prelude::IteratorRandom;
use with_error::WithError;

use crate::child_score::{ChildScoreAlgorithm, SelectionMode};

/// Plays out a game using random moves until a terminal state is reached.
///
/// Pseudocode:
/// ```text
/// 𝐟𝐮𝐧𝐜𝐭𝐢𝐨𝐧 DEFAULTPOLICY(s)
///   𝐰𝐡𝐢𝐥𝐞 s is non-terminal 𝐝𝐨
///     choose 𝒂 ∈ A(s) uniformly at random
///     s ← f(s,𝒂)
///   𝐫𝐞𝐭𝐮𝐫𝐧 reward for state s
/// ```
pub struct RandomPlayoutEvaluator {}

impl<TState: GameStateNode> StateEvaluator<TState> for RandomPlayoutEvaluator {
    fn evaluate(&self, input: &TState, player: TState::PlayerName) -> Result<i32> {
        let mut game = input.make_copy();
        loop {
            match game.status() {
                GameStatus::Completed { winner } => {
                    return Ok(if winner == player { 1 } else { -1 });
                }
                GameStatus::InProgress { current_turn } => {
                    let action = game
                        .legal_actions(current_turn)?
                        .choose(&mut rand::thread_rng())
                        .with_error(|| "No actions found")?;
                    game.execute_action(current_turn, action)?;
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
struct SearchNode<TState: GameStateNode> {
    /// Player who acted to create this node
    pub player: TState::PlayerName,
    /// Q(v): Total reward of all playouts that passed through this state
    pub total_reward: f64,
    /// N(v): Visit count for this node
    pub visit_count: u32,
}

struct SearchEdge<TState: GameStateNode> {
    pub action: TState::Action,
}

type SearchGraph<TState> = Graph<SearchNode<TState>, SearchEdge<TState>>;

/// Monte Carlo search algorithm.
///
/// Monte carlo tree search operates over a tree of game state nodes
/// connected by game actions. The search follows these three steps
/// repeatedly:
///
/// 1) **Tree Policy:** Find a node in the tree which has not previously
/// been explored. The UCT algorithm is one mathematical heuristic
/// for how to prioritize nodes to explore.
///
/// 2) **Default Policy:** Score this node to determine its reward value
/// (∆), typically by playing random moves until the game terminates.
///
/// 3) **Backpropagation:** Walk back up the tree, adding the resulting
/// reward value to each parent node.
///
/// Pseudocode:
/// ```text
/// 𝐟𝐮𝐧𝐜𝐭𝐢𝐨𝐧 UCTSEARCH(s₀)
///   create root node v₀ with state s₀
///   𝐰𝐡𝐢𝐥𝐞 within computational budget 𝐝𝐨
///     v₁ ← TREEPOLICY(v₀)
///     ∆ ← DEFAULTPOLICY(s(v₁))
///     BACKUP(v₁, ∆)
///   𝐫𝐞𝐭𝐮𝐫𝐧 𝒂(BESTCHILD(v₀, 0))
/// ```
pub struct MonteCarloAlgorithm<TScoreAlgorithm: ChildScoreAlgorithm> {
    pub child_score_algorithm: TScoreAlgorithm,
}

impl<TScoreAlgorithm: ChildScoreAlgorithm> SelectionAlgorithm
    for MonteCarloAlgorithm<TScoreAlgorithm>
{
    fn pick_action<TStateNode, TEvaluator>(
        &self,
        deadline: Instant,
        node: &TStateNode,
        evaluator: &TEvaluator,
        player: TStateNode::PlayerName,
    ) -> Result<TStateNode::Action>
    where
        TStateNode: GameStateNode,
        TEvaluator: StateEvaluator<TStateNode>,
    {
        self.run_search(|i| i % 100 == 0 && deadline < Instant::now(), node, evaluator, player)
    }
}

impl<TScoreAlgorithm: ChildScoreAlgorithm> MonteCarloAlgorithm<TScoreAlgorithm> {
    pub fn run_search<TStateNode: GameStateNode, TEvaluator: StateEvaluator<TStateNode>>(
        &self,
        should_halt: impl Fn(u32) -> bool,
        node: &TStateNode,
        evaluator: &TEvaluator,
        player: TStateNode::PlayerName,
    ) -> Result<TStateNode::Action> {
        let mut graph = SearchGraph::new();
        let root = graph.add_node(SearchNode { total_reward: 0.0, visit_count: 1, player });
        let mut i = 0;
        while !should_halt(i) {
            let mut game = node.make_copy();
            let node = self.tree_policy(&mut graph, &mut game, root)?;
            let reward = f64::from(evaluator.evaluate(&game, player)?);
            Self::backup(&mut graph, player, node, reward)?;
            i += 1;
        }

        let (action, _) = self.best_child(
            &graph,
            root,
            node.legal_actions(player)?.collect(),
            SelectionMode::Best,
        )?;
        Ok(action)
    }

    /// Returns a descendant node to examine next for the provided parent node,
    /// either:
    ///  * A node which has not yet been explored
    ///  * The best terminal node descendant, if all nodes have been explored.
    ///
    /// If possible actions are available from this node which have not yet been
    /// explored, selects an action and applies it, returning the result as a
    /// new child. Otherwise, selects the best child to explore based on
    /// visit counts and known rewards, using the [best_child] algorithm,
    /// and then repeats this process recursively until an unseen node is
    /// found (or the best child is terminal).
    ///
    /// Mutates the provided [GameState] to represent the game state at the
    /// returned node.
    ///
    /// Cᵖ is the exploration constant, Cᵖ = 1/√2 was suggested by Kocsis and
    /// Szepesvári as a good choice.
    ///
    /// Pseudocode:
    /// ```text
    /// 𝐟𝐮𝐧𝐜𝐭𝐢𝐨𝐧 TREEPOLICY(v)
    ///   𝐰𝐡𝐢𝐥𝐞 v is nonterminal 𝐝𝐨
    ///     𝐢𝐟 v not fully expanded 𝐭𝐡𝐞𝐧
    ///       𝐫𝐞𝐭𝐮𝐫𝐧 EXPAND(v)
    ///     𝐞𝐥𝐬𝐞
    ///       v ← BESTCHILD(v, Cᵖ)
    ///   𝐫𝐞𝐭𝐮𝐫𝐧 v
    /// ```
    fn tree_policy<TState: GameStateNode>(
        &self,
        graph: &mut SearchGraph<TState>,
        game: &mut TState,
        mut node: NodeIndex,
    ) -> Result<NodeIndex> {
        while let GameStatus::InProgress { current_turn } = game.status() {
            let actions = game.legal_actions(current_turn)?.collect::<HashSet<_>>();
            let explored = graph.edges(node).map(|e| e.weight().action).collect::<HashSet<_>>();
            if let Some(action) = actions.iter().find(|a| !explored.contains(a)) {
                // An action exists which has not yet been tried
                return Self::expand(graph, game, current_turn, node, *action);
            } else {
                // All actions have been tried, recursively search the best candidate
                let (action, best) =
                    self.best_child(graph, node, actions, SelectionMode::Exploration)?;
                game.execute_action(current_turn, action)?;
                node = best;
            }
        }
        Ok(node)
    }

    /// Generates a new tree node by applying the next untried action from the
    /// provided input node. Mutates the provided [GameState] to apply the
    /// provided game action.
    ///
    /// Pseudocode:
    /// ```text
    /// 𝐟𝐮𝐧𝐜𝐭𝐢𝐨𝐧 EXPAND(v)
    ///   choose 𝒂 ∈ untried actions from A(s(v))
    ///   add a new child v′ to v
    ///     with s(v′) = f(s(v), 𝒂)
    ///     and 𝒂(v′) = 𝒂
    ///   𝐫𝐞𝐭𝐮𝐫𝐧 v′
    /// ```
    fn expand<TState: GameStateNode>(
        graph: &mut SearchGraph<TState>,
        game: &mut TState,
        player: TState::PlayerName,
        source: NodeIndex,
        action: TState::Action,
    ) -> Result<NodeIndex> {
        game.execute_action(player, action)?;
        let target = graph.add_node(SearchNode { player, total_reward: 0.0, visit_count: 0 });
        graph.add_edge(source, target, SearchEdge { action });
        Ok(target)
    }

    /// Picks the most promising child node to explore, returning its associated
    /// action and node identifier.
    ///
    /// This implementation is using the UCT1 algorithm, a standard approach for
    /// selecting children and solution to the 'multi-armed bandit' problem.
    ///
    /// Pseudocode:
    /// ```text
    /// 𝐟𝐮𝐧𝐜𝐭𝐢𝐨𝐧 BESTCHILD(v,c)
    ///   𝐫𝐞𝐭𝐮𝐫𝐧 argmax(
    ///     v′ ∈ children of v:
    ///     Q(v′) / N(v′) +
    ///     c * √ [ 2 * ln(N(v)) / N(v′) ]
    ///   )
    /// ```
    fn best_child<TState: GameStateNode>(
        &self,
        graph: &SearchGraph<TState>,
        node: NodeIndex,
        legal: HashSet<TState::Action>,
        selection_mode: SelectionMode,
    ) -> Result<(TState::Action, NodeIndex)> {
        let parent_visits = graph[node].visit_count;
        let (edge, _) = graph
            .edges(node)
            // We re-check action legality here because the set of legal actions can change between
            // visits, e.g. if different cards are drawn
            .filter(|edge| legal.contains(&edge.weight().action))
            .map(|edge| {
                let child = &graph[edge.target()];
                // This can technically panic when invoked from root with a very small
                // simulation count, so don't do that :)
                assert_ne!(child.visit_count, 0);
                (
                    edge,
                    self.child_score_algorithm.score(
                        f64::from(parent_visits),
                        f64::from(child.visit_count),
                        child.total_reward,
                        selection_mode,
                    ),
                )
            })
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .with_error(|| "No children found")?;
        Ok((edge.weight().action, edge.target()))
    }

    /// Once a playout is completed, the backpropagation step walks back up the
    /// hierarchy of parent nodes, adding the resulting reward value to each
    /// one.
    ///
    /// Pseudocode:
    /// ```text
    /// 𝐟𝐮𝐧𝐜𝐭𝐢𝐨𝐧 BACKUP(v,∆)
    ///   𝐰𝐡𝐢𝐥𝐞 v is not null 𝐝𝐨
    ///     N(v) ← N(v) + 1
    ///     Q(v) ← Q(v) + ∆(v, p)
    ///     v ← parent of v
    /// ```
    fn backup<TState: GameStateNode>(
        graph: &mut SearchGraph<TState>,
        maximizing_player: TState::PlayerName,
        mut node: NodeIndex,
        reward: f64,
    ) -> Result<()> {
        loop {
            let weight = graph.node_weight_mut(node).with_error(|| "Node not found")?;
            weight.visit_count += 1;
            weight.total_reward +=
                if weight.player == maximizing_player { reward } else { -reward };

            node = match graph.neighbors_directed(node, Direction::Incoming).next() {
                Some(n) => n,
                _ => return Ok(()),
            };
        }
    }
}
