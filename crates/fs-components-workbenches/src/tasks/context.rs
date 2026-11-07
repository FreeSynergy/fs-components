//! What the workbench gives back to its host.
//!
//! Pattern: none — a workbench always gives a context back and does not run alone (ADR
//! 0056, point 4). Provisional shape: what is current, as plain values. Nothing here is a handle.

/// How the one set of tasks is shown.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TaskView {
    /// One row per task.
    #[default]
    List,
    /// One column per status; moving a card changes the status of the task.
    Status,
}

/// What is current in the workbench, for its host.
#[non_exhaustive]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TasksContext {
    /// The key of the task selected.
    pub selected: Option<String>,
    /// The view shown.
    pub view: TaskView,
}
