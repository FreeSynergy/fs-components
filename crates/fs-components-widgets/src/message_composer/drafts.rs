//! The background writer of the draft.
//!
//! Pattern: Observer (timer thread outside the composer's lock). Writes 0.5 s after the last
//! typing and at least every 5 s while typing; a failed write is tried again every 2 s and
//! reported. It stops when the composer is gone. No wall-clock behaviour is promised to callers:
//! `flush` writes at once.

use std::sync::{Arc, Mutex, PoisonError, Weak};
use std::time::{Duration, Instant};

use super::model::ComposerDrafts;

struct Pending {
    text: String,
    last: Instant,
    first: Instant,
    retry_at: Option<Instant>,
}

/// The shared part: what is waiting to be written, and the last failure.
pub(super) struct Job {
    store: Arc<dyn ComposerDrafts>,
    key: String,
    pending: Mutex<Option<Pending>>,
    pub error: Mutex<Option<String>>,
}

impl Job {
    pub fn new(store: Arc<dyn ComposerDrafts>, key: String) -> Arc<Self> {
        Arc::new(Self {
            store,
            key,
            pending: Mutex::new(None),
            error: Mutex::new(None),
        })
    }

    /// New text to keep. `changed` is called when the failure shown should be drawn again.
    pub fn touch(self: &Arc<Self>, text: &str, changed: impl Fn() + Send + 'static) {
        let start = {
            let mut p = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
            let now = Instant::now();
            let first = p.as_ref().map_or(now, |p| p.first);
            let start = p.is_none();
            *p = Some(Pending {
                text: text.to_owned(),
                last: now,
                first,
                retry_at: None,
            });
            start
        };
        if start {
            let weak = Arc::downgrade(self);
            std::thread::spawn(move || run(&weak, &changed));
        }
    }

    /// Write what is waiting now.
    pub fn flush(&self) {
        let waiting = self
            .pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        if let Some(p) = waiting {
            self.write(&p.text);
        }
    }

    /// Forget what is waiting and remove the stored draft.
    pub fn clear(&self) {
        self.pending
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take();
        let result = self.store.clear(&self.key);
        *self.error.lock().unwrap_or_else(PoisonError::into_inner) = result.err();
    }

    fn write(&self, text: &str) -> bool {
        let result = self.store.save(&self.key, text);
        let ok = result.is_ok();
        *self.error.lock().unwrap_or_else(PoisonError::into_inner) = result.err();
        ok
    }

    /// Write if it is time; says whether anything is still waiting.
    fn due(&self) -> bool {
        let mut guard = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
        let Some(p) = guard.as_ref() else {
            return false;
        };
        let now = Instant::now();
        let ready = match p.retry_at {
            Some(at) => now >= at,
            None => {
                now - p.last >= Duration::from_millis(500)
                    || now - p.first >= Duration::from_secs(5)
            }
        };
        if !ready {
            return true;
        }
        let text = p.text.clone();
        if self.write(&text) {
            *guard = None;
            return false;
        }
        if let Some(p) = guard.as_mut() {
            p.retry_at = Some(now + Duration::from_secs(2));
        }
        true
    }
}

fn run(job: &Weak<Job>, changed: &(impl Fn() + Send)) {
    loop {
        std::thread::sleep(Duration::from_millis(100));
        let Some(job) = job.upgrade() else { return };
        let before = job
            .error
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        let more = job.due();
        if before != *job.error.lock().unwrap_or_else(PoisonError::into_inner) {
            changed();
        }
        if !more {
            return;
        }
    }
}
