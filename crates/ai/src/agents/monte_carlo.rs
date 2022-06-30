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
use std::f64::consts;

use anyhow::Result;
use data::fail;
use data::game::{GamePhase, GameState};
use data::game_actions::UserAction;
use data::primitives::Side;
use data::with_error::WithError;
use ordered_float::NotNan;
use petgraph::prelude::{EdgeRef, NodeIndex};
use petgraph::{Direction, Graph};
use rand::prelude::IteratorRandom;
use rand::thread_rng;
use rules::{actions, flags};

use crate::core::legal_actions;
use crate::core::types::{notnan, StatePredictionIterator};

pub fn execute(mut states: StatePredictionIterator, side: Side) -> Result<UserAction> {
    let game = states.next().with_error(|| "Expected game state")?.state;
    uct_search(&game, side, 1000)
}

type RewardValue = NotNan<f64>;

#[derive(Debug, Clone)]
struct SearchNode {
    /// Player who acted to create this node
    pub side: Side,
    /// Q(v): Total reward of all playouts that passed through this state
    pub total_reward: RewardValue,
    /// N(v): Visit count for this node    
    pub visit_count: u32,
}

struct SearchEdge {
    pub action: UserAction,
}

type SearchGraph = Graph<SearchNode, SearchEdge>;

/// Primary UCT search function.
///
/// Monte carlo tree search operates over a tree of game state nodes connected
/// by game actions. The search follows these three steps repeatedly:
///
/// 1) **Tree Policy:** Find a node in the tree which has not previously been
/// explored. The UCT algorithm provides a mathematical heuristic for how to
/// prioritize nodes to explore.
///
/// 2) **Default Policy:** Score this node to determine its reward value (∆),
/// typically by playing random moves until the game terminates.
///
/// 3) **Backpropagation:** Walk back up the tree, adding the resulting reward
/// value to each parent node.
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
pub fn uct_search(game_state: &GameState, side: Side, simulation_steps: u32) -> Result<UserAction> {
    let mut graph = SearchGraph::new();
    let root = graph.add_node(SearchNode { total_reward: notnan(0.0), visit_count: 1, side });
    for _ in 0..simulation_steps {
        let mut game = game_state.clone();
        let node = tree_policy(&mut graph, &mut game, root)?;
        let reward = default_policy(game, side)?;
        backup(&mut graph, side, node, reward)?;
    }

    let (action, _) =
        best_child(&graph, root, legal_actions::evaluate(game_state, side).collect(), 0.0)?;
    Ok(action)
}

/// Returns a descendant node to examine next for the provided parent node,
/// either:
///  * A node which has not yet been explored
///  * The best terminal node descendant, if all nodes have been explored.
///
/// If possible actions are available from this node which have not yet been
/// explored, selects an action and applies it, returning the result as a new
/// child. Otherwise, selects the best child to explore based on visit counts
/// and known rewards, using the [best_child] algorithm, and then repeats this
/// process recursively until an unseen node is found (or the best child is
/// terminal).
///
/// Mutates the provided [GameState] to represent the game state at the returned
/// node.
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
fn tree_policy(
    graph: &mut SearchGraph,
    game: &mut GameState,
    mut node: NodeIndex,
) -> Result<NodeIndex> {
    while !matches!(game.data.phase, GamePhase::GameOver(_)) {
        let actions =
            legal_actions::evaluate(game, current_priority(game)?).collect::<HashSet<_>>();
        let explored = graph.edges(node).map(|e| e.weight().action).collect::<HashSet<_>>();
        if let Some(action) = actions.iter().find(|a| !explored.contains(a)) {
            // An action exists which has not yet been tried
            return expand(graph, game, node, *action);
        } else {
            // All actions have been tried, recursively search the best candidate
            let (action, best) = best_child(graph, node, actions, consts::FRAC_1_SQRT_2)?;
            actions::handle_user_action(game, current_priority(game)?, action)?;
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
fn expand(
    graph: &mut SearchGraph,
    game: &mut GameState,
    source: NodeIndex,
    action: UserAction,
) -> Result<NodeIndex> {
    let side = current_priority(game)?;
    // Instead of selecting an untried action here, we pass in the one we found in
    // `tree_policy` to avoid redundancy.
    actions::handle_user_action(game, side, action)?;
    let target = graph.add_node(SearchNode { side, total_reward: notnan(0.0), visit_count: 0 });
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
fn best_child(
    graph: &SearchGraph,
    node: NodeIndex,
    legal: HashSet<UserAction>,
    exploration_bias: f64,
) -> Result<(UserAction, NodeIndex)> {
    let parent_visits = graph[node].visit_count;
    let result = graph
        .edges(node)
        // We re-check action legality here because the set of legal actions can change between
        // visits, e.g. if different cards are drawn
        .filter(|edge| legal.contains(&edge.weight().action))
        .max_by_key(|edge| {
            let child = &graph[edge.target()];
            // This can technically panic when invoked from root with a very small
            // simulation count, so don't do that :)
            assert_ne!(child.visit_count, 0);
            let child_visits = f64::from(child.visit_count);
            let exploration = child.total_reward / child_visits;
            let exploitation = f64::sqrt((2.0 * f64::ln(f64::from(parent_visits))) / child_visits);
            exploration + (exploration_bias * exploitation)
        })
        .with_error(|| "No children found")?;
    Ok((result.weight().action, result.target()))
}

/// Plays out a game using random moves to determine its outcome until a
/// terminal state is reached.
///
/// Pseudocode:
/// ```text
/// 𝐟𝐮𝐧𝐜𝐭𝐢𝐨𝐧 DEFAULTPOLICY(s)
///   𝐰𝐡𝐢𝐥𝐞 s is non-terminal 𝐝𝐨
///     choose 𝒂 ∈ A(s) uniformly at random
///     s ← f(s,𝒂)
///   𝐫𝐞𝐭𝐮𝐫𝐧 reward for state s
/// ```
fn default_policy(mut game: GameState, side: Side) -> Result<RewardValue> {
    for _ in 0..60 {
        if let GamePhase::GameOver(data) = game.data.phase {
            return Ok(notnan(if data.winner == side { 10.0 } else { -10.0 }));
        }

        let side = current_priority(&game)?;
        let action = legal_actions::evaluate(&game, side)
            .choose(&mut thread_rng())
            .with_error(|| "No actions found")?;
        actions::handle_user_action(&mut game, side, action)?;
    }

    Ok(notnan(f64::from(game.player(side).score) - f64::from(game.player(side.opponent()).score)))
}

/// Once a playout is completed, the backpropagation step walks back up the
/// hierarchy of parent nodes, adding the resulting reward value to each one.
///
/// Pseudocode:
/// ```text
/// 𝐟𝐮𝐧𝐜𝐭𝐢𝐨𝐧 BACKUP(v,∆)
///   𝐰𝐡𝐢𝐥𝐞 v is not null 𝐝𝐨
///     N(v) ← N(v) + 1
///     Q(v) ← Q(v) + ∆(v, p)
///     v ← parent of v
/// ```
fn backup(
    graph: &mut SearchGraph,
    maximizing_side: Side,
    mut node: NodeIndex,
    reward: RewardValue,
) -> Result<()> {
    loop {
        let weight = graph.node_weight_mut(node).with_error(|| "Node not found")?;
        weight.visit_count += 1;
        weight.total_reward += if weight.side == maximizing_side { reward } else { -reward };

        node = match graph.neighbors_directed(node, Direction::Incoming).next() {
            Some(n) => n,
            _ => return Ok(()),
        };
    }
}

fn current_priority(game: &GameState) -> Result<Side> {
    if flags::can_take_action(game, Side::Overlord) {
        Ok(Side::Overlord)
    } else if flags::can_take_action(game, Side::Champion) {
        Ok(Side::Champion)
    } else {
        fail!("No player can take action!")
    }
}
