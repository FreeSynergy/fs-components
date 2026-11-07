//! Reading what is open.
//!
//! Pattern: State — what the detail area shows comes from the source each time it is asked for here;
//! what is gone leaves the view and says so.

use super::labels::MapLabels;
use super::shown::Shown;
use super::state::MapState;
use crate::error::ErrorKind;

impl MapState {
    /// Read what is open again, after the view moved: it may have changed elsewhere. An edit stays, and
    /// so does a question.
    pub(super) fn refresh_shown(&mut self) {
        if self.editing.is_some() {
            return;
        }
        let Some(key) = self.selected().map(str::to_string) else {
            return;
        };
        match self.source.read(&key) {
            Ok(sheet) => {
                self.shown = Shown::Place {
                    key,
                    sheet: Box::new(sheet),
                };
            }
            Err(e) if e.kind == ErrorKind::NotFound => self.place_gone(),
            Err(e) => {
                self.shown = Shown::Failed {
                    key,
                    reason: MapLabels::fill(&self.labels.unreadable, &e.message),
                };
            }
        }
    }

    /// Show `key` as it is now, ending any edit and any question.
    pub(super) fn show(&mut self, key: &str) {
        self.editing = None;
        self.delete_asked = None;
        self.notice = None;
        self.shown = match self.source.read(key) {
            Ok(sheet) => Shown::Place {
                key: key.to_string(),
                sheet: Box::new(sheet),
            },
            Err(e) if e.kind == ErrorKind::NotFound => {
                self.place_gone();
                return;
            }
            Err(e) => Shown::Failed {
                key: key.to_string(),
                reason: MapLabels::fill(&self.labels.unreadable, &e.message),
            },
        };
    }

    /// The place shown no longer exists: it leaves the view and the detail area says so.
    pub(super) fn place_gone(&mut self) {
        self.editing = None;
        self.delete_asked = None;
        self.reload_places();
        self.shown = Shown::Nothing;
        self.notice = Some(self.labels.gone.clone());
    }
}
