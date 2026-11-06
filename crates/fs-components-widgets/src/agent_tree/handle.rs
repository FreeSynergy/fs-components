//! The handle of an agent tree.
//!
//! Pattern: Facade + Observer + Memento — a clonable handle over the forest, what is closed and
//! the scroll position; no method returns a lock guard. The host feeds agents (`apply`), a
//! person's clicks come back as an [`Outcome`]. The tree decides and stops nothing.

use std::collections::HashSet;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

use super::forest::{Forest, Row};
use super::labels::Labels;
use super::model::{AgentNode, Counts, Outcome};
use crate::list_view::ListPosition;
use crate::live::Live;

/// Called with every outcome.
pub type OnOutcome = Arc<dyn Fn(&Outcome) + Send + Sync>;

struct Inner {
    forest: Forest,
    closed: HashSet<String>,
    rows: Option<Arc<Vec<Row>>>,
    on_outcome: Option<OnOutcome>,
}

/// A tree of agents. Cheap to clone; clones share everything.
#[derive(Clone)]
pub struct AgentTree {
    shared: Arc<Mutex<Inner>>,
    pub(super) live: Live,
    pub(super) position: ListPosition,
    pub(super) labels: Labels,
    pub(super) id: String,
}

impl Default for AgentTree {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentTree {
    /// No agents yet.
    pub fn new() -> Self {
        Self {
            shared: Arc::new(Mutex::new(Inner {
                forest: Forest::new(),
                closed: HashSet::new(),
                rows: None,
                on_outcome: None,
            })),
            live: Live::new(),
            position: ListPosition::new(),
            labels: Labels::default(),
            id: "fs-at".into(),
        }
    }

    fn lock(&self) -> MutexGuard<'_, Inner> {
        self.shared.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// The words.
    #[must_use]
    pub fn labels(mut self, labels: Labels) -> Self {
        self.labels = labels;
        self
    }

    /// Element id prefix (rows are `<id>-row-<key>`, toggles `<id>-toggle-<key>`).
    #[must_use]
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Hear of every outcome.
    #[must_use]
    pub fn on_outcome(self, handler: OnOutcome) -> Self {
        self.lock().on_outcome = Some(handler);
        self
    }

    /// Add an agent or replace the one with the same key, in place.
    pub fn apply(&self, node: AgentNode) -> bool {
        let new = {
            let mut inner = self.lock();
            inner.rows = None;
            inner.forest.apply(node)
        };
        self.live.refresh();
        new
    }

    /// Replace all agents by the host's list as it is now; what is closed stays closed.
    pub fn set_nodes(&self, nodes: Vec<AgentNode>) {
        {
            let mut inner = self.lock();
            inner.rows = None;
            inner.forest = Forest::new();
            for n in nodes {
                inner.forest.apply(n);
            }
        }
        self.live.refresh();
    }

    /// How many agents.
    pub fn len(&self) -> usize {
        self.lock().forest.len()
    }

    /// Are there none?
    pub fn is_empty(&self) -> bool {
        self.lock().forest.is_empty()
    }

    /// What needs attention in the whole tree, shown or not.
    pub fn counts(&self) -> Counts {
        self.lock().forest.counts()
    }

    /// The agent with this key.
    pub fn get(&self, key: &str) -> Option<AgentNode> {
        self.lock().forest.get(key).cloned()
    }

    /// Open or close a branch; reports `Toggled`.
    pub fn toggle(&self, key: &str) -> Outcome {
        let open = {
            let mut inner = self.lock();
            inner.rows = None;
            let open = inner.closed.remove(key);
            if !open {
                inner.closed.insert(key.to_owned());
            }
            open
        };
        self.report(Outcome::Toggled {
            key: key.to_owned(),
            open,
        })
    }

    /// Choose an agent; reports `Selected`.
    pub fn select(&self, key: &str) -> Outcome {
        self.report(Outcome::Selected {
            key: key.to_owned(),
        })
    }

    fn report(&self, outcome: Outcome) -> Outcome {
        let handler = self.lock().on_outcome.clone();
        if let Some(handler) = handler {
            handler(&outcome);
        }
        self.live.refresh();
        outcome
    }

    /// The lines shown now (a shared snapshot).
    pub fn rows(&self) -> Arc<Vec<Row>> {
        let mut inner = self.lock();
        if let Some(rows) = &inner.rows {
            return Arc::clone(rows);
        }
        let rows = Arc::new(inner.forest.rows(&inner.closed));
        inner.rows = Some(Arc::clone(&rows));
        rows
    }
}
