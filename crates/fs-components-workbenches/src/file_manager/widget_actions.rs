//! The callbacks of the widget.
//!
//! Pattern: none of the table fits (plain closures, no listener trait) — the callbacks the view calls when the person acts (plain closures, no operation enum); each changes the shared state under a short lock, then
//! asks for a redraw; no lock is held for either. Typing is remembered but does not redraw: the text
//! field keeps its own cursor. A file chosen in the picker is told to the host with no lock held.

use std::sync::Arc;

use super::state::FilesState;
use super::view::FileActions;
use super::widget::{FileManagerWidget, locked, rebuild};

impl FileManagerWidget {
    pub(super) fn actions(&self) -> FileActions {
        let this = self.clone();
        let (typed, typed_filter) = (Arc::clone(&self.state), Arc::clone(&self.state));
        FileActions {
            apply: Arc::new(move |change: &(dyn Fn(&mut FilesState) + Send + Sync)| {
                this.apply(change);
            }),
            typed: Arc::new(move |text: &str| locked(&typed).set_input(text)),
            typed_filter: Arc::new(move |field: &str, text: &str| {
                locked(&typed_filter).set_filter_field(field, text);
            }),
        }
    }

    /// Change the state, tell the host what was chosen, draw again.
    pub(super) fn apply(&self, change: &(dyn Fn(&mut FilesState) + Send + Sync)) {
        let (before, after) = {
            let mut held = locked(&self.state);
            let before = held.picked().cloned();
            change(&mut held);
            (before, held.picked().cloned())
        };
        if before != after
            && let Some(picked) = &after
        {
            let handler = self
                .on_pick
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone();
            if let Some(handler) = handler {
                handler(picked);
            }
        }
        rebuild();
    }
}
