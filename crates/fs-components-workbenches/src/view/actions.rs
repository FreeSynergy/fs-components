//! What the person can do, as callbacks.
//!
//! Pattern: Command-like callbacks (Observer) — the shell decides what each one does: change the
//! state and redraw (window), or nothing (picture).

use std::sync::Arc;

/// The things the view reports.
#[derive(Clone)]
pub(crate) struct Actions {
    /// Open the document at this path.
    pub open: Arc<dyn Fn(&str) + Send + Sync>,
    /// Follow this link of the document shown.
    pub follow: Arc<dyn Fn(&str) + Send + Sync>,
    /// Go back one page.
    pub back: Arc<dyn Fn() + Send + Sync>,
}

impl Actions {
    /// Callbacks that do nothing (for a picture).
    pub fn inert() -> Self {
        Self {
            open: Arc::new(|_| {}),
            follow: Arc::new(|_| {}),
            back: Arc::new(|| {}),
        }
    }
}
