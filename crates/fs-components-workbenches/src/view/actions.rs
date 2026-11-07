//! What the person can do, as callbacks.
//!
//! Pattern: Observer — the shell decides what each one does: change the
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
    /// Start editing.
    pub edit: Arc<dyn Fn() + Send + Sync>,
    /// The text of the draft changed.
    pub draft: Arc<dyn Fn(&str) + Send + Sync>,
    /// Save the draft.
    pub save: Arc<dyn Fn() + Send + Sync>,
    /// Drop the draft.
    pub cancel: Arc<dyn Fn() + Send + Sync>,
    /// Use the backed-up draft.
    pub restore: Arc<dyn Fn() + Send + Sync>,
    /// Throw the backed-up draft away.
    pub discard: Arc<dyn Fn() + Send + Sync>,
    /// Export the page.
    pub export: Arc<dyn Fn() + Send + Sync>,
}

impl Actions {
    /// Callbacks that do nothing (for a picture).
    pub fn inert() -> Self {
        Self {
            open: Arc::new(|_| {}),
            follow: Arc::new(|_| {}),
            back: Arc::new(|| {}),
            edit: Arc::new(|| {}),
            draft: Arc::new(|_| {}),
            save: Arc::new(|| {}),
            cancel: Arc::new(|| {}),
            export: Arc::new(|| {}),
            restore: Arc::new(|| {}),
            discard: Arc::new(|| {}),
        }
    }
}
