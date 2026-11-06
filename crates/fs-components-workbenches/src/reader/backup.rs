//! Backup of the draft of an edit in progress.
//!
//! Pattern: Strategy (`DraftStore`, supplied by the host) + State. The reader keeps no storage of
//! its own: a program that wants drafts to survive a crash or a closed window gives a store (a
//! file in its data directory, for example). A draft is backed up after a short pause in typing
//! and at least every few seconds while typing goes on; a failed backup is retried and shown. A
//! backup is never written back into the document by itself (ADR 0046): on the next edit of that
//! document the person is offered it and chooses Restore or Discard, and until then the offered
//! backup is not touched. A save or a chosen discard clears a backup this edit wrote. The write
//! itself can run outside the reader's lock ([`BackupJob`]), so a slow store never stalls the
//! interface.

use std::sync::Arc;
use std::time::{Duration, Instant};

use super::draft::{Draft, DraftStore};
use super::model::Reader;
use crate::error::SourceError;
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
    /// The edit (by id) whose draft has changes that are not in the store yet.
    pub dirty: Option<u64>,
    /// The edit whose draft this reader wrote into the store (and may clear again).
    pub written: Option<u64>,
    pub last_change: Instant,
    pub last_attempt: Instant,
    pub problem: Option<String>,
}

impl Backup {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            store: None,
            timing: Timing {
                pause: Duration::from_millis(500),
                at_most: Duration::from_secs(5),
            },
            dirty: None,
            written: None,
            last_change: now,
            last_attempt: now,
            problem: None,
        }
    }
}

/// One write to the store, made under the reader's lock and run outside it.
pub struct BackupJob {
    store: Arc<dyn DraftStore>,
    path: String,
    edit: u64,
    draft: Option<Draft>,
}

impl BackupJob {
    /// Do the write (may be slow; do not hold the reader's lock).
    pub fn run(&self) -> Result<(), SourceError> {
        match &self.draft {
            Some(d) => self.store.save(d),
            None => self.store.clear(&self.path),
        }
    }
}

/// How long a failed backup waits before it is tried again.
const RETRY: Duration = Duration::from_secs(2);

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
        if let Some(e) = &self.editing {
            self.backup.dirty = Some(e.id);
            self.backup.last_change = Instant::now();
        }
    }

    /// The write that is due now, if any: the draft changed, typing paused or the last attempt is
    /// old, and no backup is waiting for the person's answer. Run it with [`BackupJob::run`]
    /// (outside the lock) and hand the result to [`Reader::backup_done`].
    pub fn backup_job(&mut self) -> Option<BackupJob> {
        let b = &self.backup;
        let due = |paused: bool| {
            let since = b.last_attempt.elapsed();
            let wait = if b.problem.is_some() {
                RETRY
            } else {
                b.timing.pause
            };
            (paused && since >= wait) || since >= b.timing.at_most.max(wait)
        };
        let e = self.editing.as_ref()?;
        if b.dirty != Some(e.id) || e.backup_offer.is_some() {
            return None;
        }
        if !due(b.last_change.elapsed() >= b.timing.pause) {
            return None;
        }
        self.make_job(false)
    }

    /// The write for the draft now, due or not (the offered backup is still not touched).
    pub fn backup_job_now(&mut self) -> Option<BackupJob> {
        let e = self.editing.as_ref()?;
        if self.backup.dirty != Some(e.id) || e.backup_offer.is_some() {
            return None;
        }
        self.make_job(true)
    }

    fn make_job(&mut self, _now: bool) -> Option<BackupJob> {
        let store = self.backup.store.clone()?;
        let path = self.current_path()?.to_string();
        let e = self.editing.as_ref()?;
        let draft = if e.changed() {
            Some(Draft::new(&path, &e.draft, &e.original, e.revision.clone()))
        } else if self.backup.written == Some(e.id) {
            None // typed and undone: remove what this edit wrote
        } else {
            self.backup.dirty = None;
            return None;
        };
        Some(BackupJob {
            store,
            path,
            edit: e.id,
            draft,
        })
    }

    /// Record the result of a [`BackupJob`]. A failure keeps the draft marked as not backed up,
    /// so it is tried again, and is shown.
    pub fn backup_done(&mut self, job: &BackupJob, result: Result<(), SourceError>) {
        self.backup.last_attempt = Instant::now();
        match result {
            Ok(()) => {
                self.backup.problem = None;
                self.backup.written = job.draft.is_some().then_some(job.edit);
                if self.backup.dirty == Some(job.edit) && !self.changed_since(job) {
                    self.backup.dirty = None;
                }
            }
            Err(e) => {
                self.backup.problem = Some(Labels::fill(&self.labels.backup_failed, &e.message));
            }
        }
    }

    /// Did the draft change after the job was made (more typing during the write)?
    fn changed_since(&self, job: &BackupJob) -> bool {
        match (&self.editing, &job.draft) {
            (Some(e), Some(d)) => e.draft != d.text,
            _ => false,
        }
    }

    /// Back the draft up if it is due (the write runs under the caller's lock; a shell that cares
    /// uses [`Reader::backup_job`] instead).
    pub fn backup_if_due(&mut self) {
        if let Some(job) = self.backup_job() {
            let result = job.run();
            self.backup_done(&job, result);
        }
    }

    /// Back the draft up now.
    pub fn backup_now(&mut self) {
        if let Some(job) = self.backup_job_now() {
            let result = job.run();
            self.backup_done(&job, result);
        }
    }
}

impl Drop for Reader {
    /// Best effort when a reader is dropped (a window's global state may keep it alive instead,
    /// then the timer has already written the draft).
    fn drop(&mut self) {
        self.backup_now();
    }
}
