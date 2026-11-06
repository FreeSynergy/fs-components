//! Backup of the draft of an edit in progress.
//!
//! Pattern: Strategy (`DraftStore`, supplied by the host) + State. The reader keeps no storage of
//! its own: a program that wants drafts to survive a crash or a closed window gives a store (a
//! file in its data directory, for example). A backup is written after a short pause in typing
//! and at most every few seconds while typing goes on, and when the reader is dropped; it is
//! never written back into the document by itself (ADR 0046): on the next edit of that document
//! the person is offered the draft and chooses to restore or discard it. A save and a chosen
//! discard clear it. A failing store is shown, not hidden.

use std::sync::Arc;
use std::time::{Duration, Instant};

use super::draft::{Draft, DraftStore};
use super::edit::Editing;
use super::model::Reader;
use crate::labels::Labels;

/// When to back up.
#[derive(Clone, Copy, Debug)]
pub(super) struct Timing {
    /// Back up after this long without typing.
    pub pause: Duration,
    /// While typing goes on, back up at least this often.
    pub at_most: Duration,
}

/// What the reader knows about backups.
pub(super) struct Backup {
    pub store: Option<Arc<dyn DraftStore>>,
    pub timing: Timing,
    pub dirty: bool,
    pub last_change: Instant,
    pub last_written: Instant,
    pub problem: Option<String>,
}

impl Backup {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            store: None,
            timing: Timing {
                pause: Duration::from_secs(1),
                at_most: Duration::from_secs(10),
            },
            dirty: false,
            last_change: now,
            last_written: now,
            problem: None,
        }
    }
}

impl Reader {
    /// Back drafts up into `store` (none = no backup).
    pub fn set_draft_store(&mut self, store: Option<Arc<dyn DraftStore>>) {
        self.backup.store = store;
    }

    /// Back up after `pause` without typing, and at least every `at_most` while typing goes on.
    pub fn set_backup_timing(&mut self, pause: Duration, at_most: Duration) {
        self.backup.timing = Timing { pause, at_most };
    }

    /// Why the last backup, restore or clear failed, if it did.
    pub fn backup_problem(&self) -> Option<&str> {
        self.backup.problem.as_deref()
    }

    /// Remember that the draft changed (called by `set_draft`).
    pub(super) fn note_change(&mut self) {
        self.backup.dirty = true;
        self.backup.last_change = Instant::now();
    }

    /// Back the draft up if it is due: changed, and typing paused or the last backup old. The
    /// shell calls this every second or so.
    pub fn backup_if_due(&mut self) {
        let b = &self.backup;
        let due = b.dirty
            && (b.last_change.elapsed() >= b.timing.pause
                || b.last_written.elapsed() >= b.timing.at_most);
        if due {
            self.backup_now();
        }
    }

    /// Back the draft up now (a changed draft only; an unchanged one removes the backup).
    pub fn backup_now(&mut self) {
        let (Some(store), Some(path), Some(e)) = (
            self.backup.store.clone(),
            self.current_path().map(str::to_string),
            self.editing.as_ref(),
        ) else {
            return;
        };
        let result = if e.changed() {
            store.save(&Draft::new(
                &path,
                &e.draft,
                &e.original,
                e.revision.clone(),
            ))
        } else {
            store.clear(&path)
        };
        self.backup.dirty = false;
        self.backup.last_written = Instant::now();
        self.backup.problem = result
            .err()
            .map(|e| Labels::fill(&self.labels.backup_failed, &e.message));
    }

    /// Look for a backup of the document being edited and offer it.
    pub(super) fn find_backup(&mut self, path: &str, editing: &mut Editing) {
        let Some(store) = self.backup.store.clone() else {
            return;
        };
        match store.load(path) {
            Ok(Some(d)) if d.text != editing.original => editing.backup_offer = Some(d),
            Ok(_) => {}
            Err(e) => {
                self.backup.problem = Some(Labels::fill(&self.labels.backup_failed, &e.message));
            }
        }
    }

    /// Use the backed-up draft. If the document changed since, the draft keeps the revision it
    /// was made from, so saving reports the conflict like any other (and a second Save replaces
    /// the newer version knowingly).
    pub fn restore_backup(&mut self) {
        let stale = self.editing.as_ref().and_then(|e| {
            let offer = e.backup_offer.as_ref()?;
            (offer.revision != e.revision || offer.original != e.original).then_some(())
        });
        let Some(editing) = &mut self.editing else {
            return;
        };
        let Some(offer) = editing.backup_offer.take() else {
            return;
        };
        editing.draft = offer.text;
        editing.discard_asked = false;
        if stale.is_some() {
            editing.revision = offer.revision;
            editing.error = Some(self.labels.backup_stale.clone());
        }
        self.note_change();
    }

    /// Throw the backup away (the person chose to).
    pub fn discard_backup(&mut self) {
        if let Some(e) = &mut self.editing {
            e.backup_offer = None;
        }
        self.clear_backup();
    }

    /// Remove the backup of the document shown (after a save or a chosen discard).
    pub(super) fn clear_backup(&mut self) {
        let (Some(store), Some(path)) = (self.backup.store.clone(), self.current_path()) else {
            return;
        };
        let result = store.clear(path);
        self.backup.dirty = false;
        self.backup.problem = result
            .err()
            .map(|e| Labels::fill(&self.labels.backup_failed, &e.message));
    }
}

impl Drop for Reader {
    /// Closing the window drops the reader: last chance to back the draft up.
    fn drop(&mut self) {
        if self.backup.dirty {
            self.backup_now();
        }
    }
}
