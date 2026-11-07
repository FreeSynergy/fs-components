//! Saving an edit.
//!
//! Pattern: State — a failed save keeps the draft and shows why, and after a conflict the next Save
//! writes knowingly.

use super::labels::MapLabels;
use super::state::MapState;
use crate::error::{ErrorKind, SourceError};

impl MapState {
    /// Save. Without changes nothing is written. On success the place is read again and shown; on
    /// failure, also a conflict, the draft stays and the reason is shown.
    pub fn save(&mut self) {
        let Some(edit) = &self.editing else {
            return;
        };
        if !edit.draft.changed() {
            let key = edit.key.clone();
            self.editing = None;
            if let Some(key) = key {
                self.show(&key);
            }
            self.notice = Some(self.labels.no_changes.clone());
            return;
        }
        let (key, draft) = (edit.key.clone(), edit.draft.clone());
        let done = match &key {
            Some(key) => self.source.save(key, &draft).map(|()| key.clone()),
            None => self.source.create(&draft),
        };
        match done {
            Ok(saved) => {
                let message = if key.is_some() {
                    self.labels.saved.clone()
                } else {
                    self.labels.created.clone()
                };
                self.show(&saved);
                self.reload_places();
                self.notice = Some(message);
            }
            Err(e) if e.kind == ErrorKind::NotFound => {
                // What the person typed is theirs: the place leaves the view, the draft stays.
                self.reload_places();
                let gone = self.labels.gone_draft.clone();
                if let Some(edit) = &mut self.editing {
                    edit.error = Some(gone);
                }
            }
            Err(e) => self.save_failed(key.as_deref(), &e),
        }
    }

    fn save_failed(&mut self, key: Option<&str>, e: &SourceError) {
        // Whoever got in the way may have changed the list: it shows as it is now.
        self.reload_places();
        let conflict = e.kind == ErrorKind::Conflict;
        let reason = if conflict {
            MapLabels::fill(&self.labels.conflict_confirm, &e.message)
        } else {
            MapLabels::fill(&self.labels.not_saved, &e.message)
        };
        // After a conflict the next Save, with the revision found now, writes over the other change:
        // the person has been told and has the draft in view.
        let latest = key
            .filter(|_| conflict)
            .and_then(|k| self.source.draft(k).ok())
            .map(|d| d.revision);
        if let Some(edit) = &mut self.editing {
            edit.error = Some(reason);
            if let Some(revision) = latest {
                edit.draft.revision = revision;
            }
        }
    }
}
