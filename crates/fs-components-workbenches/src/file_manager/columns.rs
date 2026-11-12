//! The columns of the list as a person arranges them: shown, hidden, in order, wide or narrow.
//!
//! Pattern: Memento over `ColumnLayout` of `list-view` — the layout is plain text kept by a `LayoutStore` under the key
//! of this view (ADR 0069: the choice, never a position), read when the store is given and written at every change.

use std::sync::Arc;

use fs_components_widgets::{ColumnLayout, LayoutStore};

use super::state::FilesState;

/// The names of the columns, in the order the caller shows them; the name cannot be hidden.
pub(super) const KEYS: [&str; 4] = ["name", "size", "modified", "type"];

pub(super) fn default_columns() -> ColumnLayout {
    ColumnLayout::new(&KEYS).keep_shown("name")
}

impl FilesState {
    /// The layout of the columns that holds now.
    pub fn columns(&self) -> &ColumnLayout {
        &self.columns
    }

    /// The person changed the columns: take it, and keep it if there is a store.
    pub fn set_columns(&mut self, layout: ColumnLayout) {
        if let Some((store, view)) = &self.column_store {
            store.save(view, &layout.to_text());
        }
        self.columns = layout;
    }

    /// Keep the layout of the columns in `store` under `view` (the key of this view), and take what it holds: the
    /// order, the hidden columns and the widths come back after a restart.
    pub fn keep_columns_in(&mut self, store: Arc<dyn LayoutStore>, view: &str) {
        if let Some(text) = store.load(view) {
            self.columns = default_columns().with_text(&text);
        }
        self.column_store = Some((store, view.to_string()));
    }
}
