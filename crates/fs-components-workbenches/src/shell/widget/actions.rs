//! The callbacks of the reader widget: they change the shared reader and ask the areas to draw again.
//!
//! Pattern: Command table — one closure per thing the person does.

use std::sync::{Arc, PoisonError};

use fs_components_widgets::open_external;

use super::{ReaderWidget, bump, locked};
use crate::reader::{Outcome, Reader};
use crate::view::Actions;

impl ReaderWidget {
    /// The callbacks that change the shared reader and ask the areas to draw again.
    pub(super) fn actions(&self) -> Actions {
        let id = self.id;
        let simple = |change: fn(&mut Reader)| -> Arc<dyn Fn() + Send + Sync> {
            let reader = Arc::clone(&self.reader);
            Arc::new(move || {
                change(&mut locked(&reader));
                bump(id, "page");
                bump(id, "side");
            })
        };
        let (open_r, follow_r, draft_r, block_r) = (
            Arc::clone(&self.reader),
            Arc::clone(&self.reader),
            Arc::clone(&self.reader),
            Arc::clone(&self.reader),
        );
        let tree_t = Arc::clone(&self.tree);
        Actions {
            open: Arc::new(move |path| {
                locked(&open_r).open(path);
                bump(id, "page");
                bump(id, "side"); // the tree marks the page that is now open
            }),
            toggle: Arc::new(move |key| {
                tree_t
                    .lock()
                    .unwrap_or_else(PoisonError::into_inner)
                    .toggle(key);
                bump(id, "side");
            }),
            block: Arc::new(move |_| {
                locked(&block_r).begin_edit();
                bump(id, "page");
            }),
            follow: Arc::new(move |href| {
                let outcome = locked(&follow_r).follow(href);
                if let Outcome::External(url) = outcome {
                    open_external(&url);
                }
                bump(id, "page");
                bump(id, "side"); // the tree marks the page that is now open
            }),
            back: simple(Reader::back), // draws both areas (the tree marks the page)
            edit: simple(Reader::begin_edit),
            // Typing is remembered but does not redraw: the text area keeps its own cursor.
            draft: Arc::new(move |text| locked(&draft_r).set_draft(text)),
            save: simple(Reader::save),
            cancel: simple(Reader::cancel_edit),
            export: simple(Reader::export),
            restore: simple(Reader::restore_backup),
            discard: simple(Reader::discard_backup),
        }
    }
}
