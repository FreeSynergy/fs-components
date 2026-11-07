//! The agents as a forest, flattened for drawing without recursion.
//!
//! Pattern: Repository (in memory, by key) + one pure pass that flattens it. Nothing recurses, so a tree
//! ten thousand deep cannot overflow the stack. Nothing is dropped: an agent whose parent is
//! unknown, or that sits in a cycle, is shown as a top agent.

use std::collections::{HashMap, HashSet};

use super::model::{AgentNode, Attention, Counts};

/// One line of the flattened tree.
#[non_exhaustive]
#[derive(Clone, Debug)]
pub struct Row {
    /// The agent.
    pub node: AgentNode,
    /// How deep (0 = top).
    pub depth: usize,
    /// Does it have agents under it?
    pub has_children: bool,
    /// Is it closed?
    pub collapsed: bool,
    /// What its own state asks.
    pub attention: Option<Attention>,
    /// What all agents under it ask (also when they are not shown).
    pub below: Counts,
}

/// The agents, in the order they first appeared, each once.
#[derive(Clone, Debug, Default)]
pub struct Forest {
    nodes: Vec<AgentNode>,
    index: HashMap<String, usize>,
}

impl Forest {
    /// No agents.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an agent or replace the one with the same key, where it stood. True if it is new.
    pub fn apply(&mut self, node: AgentNode) -> bool {
        if let Some(&i) = self.index.get(&node.key) {
            self.nodes[i] = node;
            return false;
        }
        self.index.insert(node.key.clone(), self.nodes.len());
        self.nodes.push(node);
        true
    }

    /// How many agents.
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Are there none?
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// The agent with this key.
    pub fn get(&self, key: &str) -> Option<&AgentNode> {
        self.index.get(key).map(|&i| &self.nodes[i])
    }

    /// The agents in the order they first appeared.
    pub fn nodes(&self) -> &[AgentNode] {
        &self.nodes
    }

    /// How many agents are working.
    pub fn working(&self) -> usize {
        self.nodes
            .iter()
            .filter(|n| n.state == super::model::AgentState::Working)
            .count()
    }

    /// What every agent asks, all together.
    pub fn counts(&self) -> Counts {
        let mut c = Counts::default();
        for n in &self.nodes {
            c.count(n.attention());
        }
        c
    }

    /// The tree as lines, in order, without the agents under a closed one.
    pub fn rows(&self, collapsed: &HashSet<String>) -> Vec<Row> {
        let n = self.nodes.len();
        let mut children: Vec<Vec<usize>> = vec![Vec::new(); n];
        let mut roots = Vec::new();
        for (i, node) in self.nodes.iter().enumerate() {
            match node.parent.as_ref().and_then(|p| self.index.get(p)) {
                Some(&p) if p != i => children[p].push(i),
                _ => roots.push(i),
            }
        }
        // preorder, depth and the parent each agent was reached from; no recursion
        let (mut order, mut seen) = (Vec::with_capacity(n), vec![false; n]);
        let (mut depth, mut up) = (vec![0usize; n], vec![usize::MAX; n]);
        let mut walk = |start: usize, order: &mut Vec<usize>, seen: &mut Vec<bool>| {
            let mut stack = vec![start];
            while let Some(i) = stack.pop() {
                if std::mem::replace(&mut seen[i], true) {
                    continue;
                }
                order.push(i);
                for &c in children[i].iter().rev() {
                    if !seen[c] {
                        depth[c] = depth[i] + 1;
                        up[c] = i;
                        stack.push(c);
                    }
                }
            }
        };
        for &r in &roots {
            walk(r, &mut order, &mut seen);
        }
        for i in 0..n {
            if !seen[i] {
                walk(i, &mut order, &mut seen); // a cycle: shown as a top agent
            }
        }
        let mut below = vec![Counts::default(); n];
        let mut kids = vec![0usize; n];
        for &i in order.iter().rev() {
            if up[i] != usize::MAX {
                let mut mine = below[i];
                mine.count(self.nodes[i].attention());
                below[up[i]].add(mine);
                kids[up[i]] += 1;
            }
        }
        let mut rows = Vec::new();
        let mut skip_below: Option<usize> = None;
        for &i in &order {
            if let Some(d) = skip_below {
                if depth[i] > d {
                    continue;
                }
                skip_below = None;
            }
            let node = &self.nodes[i];
            let closed = kids[i] > 0 && collapsed.contains(&node.key);
            if closed {
                skip_below = Some(depth[i]);
            }
            rows.push(Row {
                node: node.clone(),
                depth: depth[i],
                has_children: kids[i] > 0,
                collapsed: closed,
                attention: node.attention(),
                below: below[i],
            });
        }
        rows
    }
}
