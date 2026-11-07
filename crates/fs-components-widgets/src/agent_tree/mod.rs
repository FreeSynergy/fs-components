//! `agent-tree`: agents and subagents, their states, and what needs a person.
//!
//! Pattern: none — this file only declares modules and re-exports. The parts: `handle` (a handle over
//! state and scroll position, with the outcome callback) and `view` (Blinc, fixed-height rows, only
//! the rows in view exist). Pure `Forest` flattens the tree without
//! recursion. Attention is only a question, a delivery or a failure; a closed branch says that
//! something below needs attention, and a line above the tree counts everything. The tree decides
//! and stops nothing: clicks are outcomes; an emergency stop is the engine's job (ADR 0054).

mod forest;
mod handle;
mod labels;
mod model;
mod styles;
mod view;

pub use forest::{Forest, Row};
pub use handle::{AgentTree, OnOutcome};
pub use labels::Labels;
pub use model::{AgentEnd, AgentNode, AgentState, Attention, Counts, Outcome};
pub use styles::STYLES;
pub use view::ROW_HEIGHT;
