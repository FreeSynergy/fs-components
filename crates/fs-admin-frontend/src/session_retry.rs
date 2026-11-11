//! "Try again" after a failed handshake, as two halves.
//!
//! Pattern: part of [`Session`] (State), split for the file limit. Everything that can take time (connecting again,
//! reading the records, asking the core what it can do) happens between the halves, in [`RetryJob::run`], outside the
//! session: it holds no lock and runs on a worker. Only the answers are put into the session, under its lock.

use std::collections::BTreeMap;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Arc;

use crate::backend::{Backend, BackendError, Record, Values};
use crate::session::Session;

/// What a try needs, taken from the session so that the work needs no lock.
pub struct RetryJob {
    backend: Arc<dyn Backend>,
    query: Values,
    /// Each command with whether its description says it is supported.
    commands: Vec<(String, bool)>,
}

/// What a try found out, to be put into the session by [`Session::retry_finish`].
pub struct RetryOutcome {
    connected: Result<(), BackendError>,
    read: Option<(Result<Vec<Record>, BackendError>, BTreeMap<String, bool>)>,
}

impl RetryOutcome {
    /// A try that could not even be started (no thread to run it on).
    #[must_use]
    pub fn failed(error: BackendError) -> Self {
        Self {
            connected: Err(error),
            read: None,
        }
    }
}

impl RetryJob {
    /// Connect again, and if that worked read the records and ask the core about its commands. A backend that panics
    /// is a failed try (`Unavailable`), never a try that does not end: the session would then refuse every later
    /// "Try again".
    #[must_use]
    pub fn run(self) -> RetryOutcome {
        let connected = catch_unwind(AssertUnwindSafe(|| self.backend.reconnect()))
            .unwrap_or_else(|_| Err(BackendError::Unavailable("reconnect panicked".into())));
        if connected.is_err() {
            return RetryOutcome {
                connected,
                read: None,
            };
        }
        let guarded = catch_unwind(AssertUnwindSafe(|| {
            let supported = self
                .commands
                .iter()
                .map(|(name, described)| (name.clone(), *described && self.backend.supported(name)))
                .collect();
            (self.backend.list(&self.query), supported)
        }));
        match guarded {
            Ok(read) => RetryOutcome {
                connected,
                read: Some(read),
            },
            Err(_) => RetryOutcome {
                connected: Err(BackendError::Unavailable("reading panicked".into())),
                read: None,
            },
        }
    }
}

impl Session {
    /// "Try again" after a failed handshake, all in one go (a test, or a caller that may wait). A window uses
    /// [`Session::retry_begin`], [`RetryJob::run`] and [`Session::retry_finish`] so that nothing slow holds the
    /// session or the interface thread.
    pub fn retry(&mut self) -> bool {
        match self.retry_begin() {
            Some(job) => {
                let outcome = job.run();
                self.retry_finish(outcome)
            }
            None => false,
        }
    }

    /// First half of "Try again": what the work needs, to be run with no lock held; `None` when a try is already
    /// running (a second press does nothing).
    pub fn retry_begin(&mut self) -> Option<RetryJob> {
        if self.retrying {
            return None;
        }
        self.retrying = true;
        Some(RetryJob {
            backend: Arc::clone(&self.backend),
            query: self.query.clone(),
            commands: self
                .plans
                .iter()
                .map(|p| (p.name.clone(), p.described_supported))
                .collect(),
        })
    }

    /// Second half: put the answers in. A failure is told again (once), with its own words; success closes the
    /// message and shows the records read.
    pub fn retry_finish(&mut self, outcome: RetryOutcome) -> bool {
        self.retrying = false;
        match outcome {
            RetryOutcome {
                connected: Ok(()),
                read: Some((read, supported)),
            } => {
                self.messages.solve("handshake");
                self.apply_list(read, Some(supported))
            }
            RetryOutcome { connected, .. } => {
                let e = connected
                    .err()
                    .unwrap_or_else(|| BackendError::Unavailable("the try gave no answer".into()));
                self.stale = matches!(e, BackendError::Unavailable(_) | BackendError::Handshake(_));
                self.fail(&e);
                false
            }
        }
    }
}
