//! `agent-tree`: agents and subagents, their states, and what needs a person.
//!
//! Pattern: Facade + Observer + Memento (handle over state and scroll position) + Adapter (Blinc,
//! fixed-height rows, only the rows in view exist). Pure `Forest` flattens the tree without
//! recursion. Attention is only a question, a delivery or a failure; a closed branch says that
//! something below needs attention, and a line above the tree counts everything. The tree decides
//! and stops nothing: clicks are outcomes; an emergency stop is the engine's job (ADR 0054).

mod forest;
mod handle;
mod labels;
mod model;
mod view;

pub use forest::{Forest, Row};
pub use handle::{AgentTree, OnOutcome};
pub use labels::Labels;
pub use model::{AgentEnd, AgentNode, AgentState, Attention, Counts, Outcome};
pub use view::ROW_HEIGHT;

/// CSS of the agent tree: only theme variables.
pub const STYLES: &str = r"
.fs-at { display: flex; flex-direction: column; background: var(--surface); border-width: 1px; border-color: var(--border); border-radius: var(--radius-default); overflow: hidden; }
.fs-at-head { display: flex; flex-direction: row; align-items: center; flex-shrink: 0; padding: 0 var(--space-3); border-width: 0px; background: var(--surface-elevated); }
.fs-at-head-text, .fs-at-name { color: var(--text-primary); font-size: var(--text-sm); }
.fs-at-row { display: flex; flex-direction: row; align-items: center; gap: var(--space-2); padding: 0 var(--space-2); overflow: hidden; border-left-width: 4px; border-color: var(--surface); }
.fs-at-toggle { display: flex; flex-direction: row; justify-content: center; flex-shrink: 0; width: 20px; }
.fs-at-main { min-width: 72px; display: flex; flex-direction: row; align-items: center; gap: var(--space-2); flex-grow: 1; min-width: 0; overflow: hidden; }
.fs-at-summary, .fs-at-meta { color: var(--text-secondary); font-size: var(--text-xs); }
.fs-at-state { color: var(--text-secondary); font-size: var(--text-xs); flex-shrink: 0; }
.fs-at-below { flex-shrink: 1; min-width: 0; overflow: hidden; padding: 0 var(--space-2); border-radius: var(--radius-sm); border-width: 1px; border-color: var(--border); }
.fs-at-question { background: var(--warning-bg); border-color: var(--warning); }
.fs-at-failure { background: var(--error-bg); border-color: var(--error); }
.fs-at-delivery { background: var(--success-bg); border-color: var(--success); }
.fs-at-empty { padding: var(--space-4); color: var(--text-secondary); }
";
