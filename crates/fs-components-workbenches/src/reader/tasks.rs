//! Ticking a checkbox in the reading view.
//!
//! Pattern: Command — the reader asks the source to change one box; the source stores it as an ordinary change.

use super::model::Reader;
use super::state::Shown;
use crate::labels::Labels;

impl Reader {
    /// Can the checkboxes of the document shown be ticked (no right: they are drawn locked)?
    pub fn can_tick(&self) -> bool {
        self.editing().is_none() && self.current_path().is_some_and(|p| self.source.can_tick(p))
    }

    /// Set checkbox `index` (document order, see `DocumentSource::set_task`) to `done`: the source stores it, the document
    /// is read again. Without the right nothing is asked of the source. A failure is shown as a notice and changes nothing.
    pub fn toggle_task(&mut self, index: usize, done: bool) {
        if !self.can_tick() {
            return;
        }
        let Some(path) = self.current_path().map(str::to_string) else {
            return;
        };
        let revision = match &self.shown {
            Shown::Document { document, .. } => document.revision.clone(),
            _ => None,
        };
        match self
            .source
            .set_task(&path, index, done, revision.as_deref())
        {
            Ok(()) => {
                self.show(&path);
                self.refresh_entries();
            }
            Err(e) => self.notice = Some(Labels::fill(&self.labels.not_saved, &e.message)),
        }
    }
}
