//! Answering an offered backup: look for it, restore it, discard it, clear what an edit wrote.
//!
//! Pattern: State transitions on `Reader` (see `backup.rs` for the timing and the writes).

use super::backup::pending_key;
use super::draft::Draft;
use super::edit::Editing;
use super::model::Reader;
use crate::labels::Labels;

impl Reader {
    /// Look for a backup of the document being edited and offer it. A backup that is not for this
    /// document (a store that shares a file between lossy names) is reported, never offered; one
    /// that equals the document is removed.
    pub(super) fn find_backup(&mut self, path: &str, editing: &mut Editing) {
        let Some(store) = self.backup.store.clone() else {
            return;
        };
        // Text typed while an earlier offer was unanswered is newer: offer that.
        let pending = pending_key(path);
        if let Ok(Some(d)) = store.load(&pending)
            && d.path == pending
            && d.text != editing.original
        {
            editing.backup_offer = Some(Draft::new(path, d.text, d.original, d.revision));
            return;
        }
        match store.load(path) {
            Ok(Some(d)) if d.path != path => {
                self.backup.problem = Some(Labels::fill(&self.labels.backup_mismatch, &d.path));
            }
            Ok(Some(d)) if d.text == editing.original => {
                if let Err(e) = store.clear(path) {
                    self.backup.problem =
                        Some(Labels::fill(&self.labels.backup_failed, &e.message));
                }
            }
            Ok(Some(d)) => editing.backup_offer = Some(d),
            Ok(None) => {}
            Err(e) => {
                self.backup.problem = Some(Labels::fill(&self.labels.backup_failed, &e.message));
            }
        }
    }

    /// Use the backed-up draft. If the person has typed something, the first call only asks. If
    /// the document changed since the draft was made, the first Save reports it before it writes
    /// (through the source's revision, or, when the source has none, by the reader).
    pub fn restore_backup(&mut self) {
        let path = self.current_path().map(str::to_string);
        let Some(editing) = &mut self.editing else {
            return;
        };
        if editing.backup_offer.is_none() {
            return;
        }
        if editing.changed() && !editing.restore_asked {
            editing.restore_asked = true;
            self.notice = Some(self.labels.restore_confirm.clone());
            return;
        }
        let Some(offer) = editing.backup_offer.take() else {
            return;
        };
        let editing_id = editing.id;
        let changed_doc = offer.original != editing.original || offer.revision != editing.revision;
        editing.draft = offer.text;
        editing.discard_asked = false;
        editing.restore_asked = false;
        if changed_doc {
            if editing.revision.is_some() && offer.revision.is_some() {
                editing.revision = offer.revision; // the source reports the conflict on Save
            } else {
                editing.stale = true; // no revisions to compare: the reader asks once
            }
            editing.error = Some(self.labels.backup_stale.clone());
        }
        self.notice = None;
        // The restored text is this edit's now: a save or a chosen discard clears the backup, and
        // the second slot (text typed while the offer was open) is obsolete.
        if let (Some(store), Some(path)) = (self.backup.store.clone(), path) {
            let _ = store.clear(&pending_key(&path));
            self.backup.written = Some((editing_id, path));
        }
        self.backup.generation += 1;
        self.note_change();
    }

    /// Throw the offered backup away (the person chose to). What was typed since stays and is
    /// backed up from now on.
    pub fn discard_backup(&mut self) {
        let Some(path) = self.current_path().map(str::to_string) else {
            return;
        };
        let Some(editing) = &mut self.editing else {
            return;
        };
        if editing.backup_offer.take().is_none() {
            return;
        }
        editing.restore_asked = false;
        let typed = editing.changed();
        if let Some(store) = self.backup.store.clone() {
            let _ = store.clear(&pending_key(&path));
            self.backup.problem = store
                .clear(&path)
                .err()
                .map(|e| Labels::fill(&self.labels.backup_failed, &e.message));
        }
        self.backup.written = None;
        self.backup.generation += 1;
        if typed {
            self.note_change();
        }
    }

    /// Remove the backup that edit `edit` of the document at `path` wrote (after a save or a
    /// chosen discard). A backup this edit did not write (an offered one) stays.
    pub(super) fn clear_own_backup(&mut self, path: &str, edit: u64) {
        self.backup.generation += 1;
        let Some((_, key)) = self.backup.written.clone().filter(|w| w.0 == edit) else {
            return;
        };
        let Some(store) = self.backup.store.clone() else {
            return;
        };
        let _ = path;
        self.backup.problem = store
            .clear(&key)
            .err()
            .map(|e| Labels::fill(&self.labels.backup_failed, &e.message));
        self.backup.written = None;
        self.backup.dirty = None;
    }
}
