//! "Try again" after a failed handshake, as two halves.
//!
//! Pattern: part of [`Session`] (State), split for the file limit. The connecting happens between the halves, outside
//! the session, so it holds no lock and runs on a worker.

use std::sync::Arc;

use crate::backend::{Backend, BackendError};
use crate::session::Session;

impl Session {
    /// "Try again" after a failed handshake, all in one go (a test, or a caller that may wait): connect again, then
    /// read the records again. A window uses [`Session::retry_begin`] and [`Session::retry_finish`] so that the
    /// connecting does not hold the session or the interface thread.
    pub fn retry(&mut self) -> bool {
        match self.retry_begin() {
            Some(backend) => {
                let connected = backend.reconnect();
                self.retry_finish(connected)
            }
            None => false,
        }
    }

    /// First half of "Try again": the backend to connect again, to be called with no lock held; `None` when a try is
    /// already running (a second press does nothing).
    pub fn retry_begin(&mut self) -> Option<Arc<dyn Backend>> {
        if self.retrying {
            return None;
        }
        self.retrying = true;
        Some(Arc::clone(&self.backend))
    }

    /// Second half: what `Backend::reconnect` answered. A failure is told again (once), with its own words; success
    /// closes the message and reads the records again.
    pub fn retry_finish(&mut self, connected: Result<(), BackendError>) -> bool {
        self.retrying = false;
        match connected {
            Ok(()) => {
                self.messages.solve("handshake");
                self.refresh_support();
                self.reload()
            }
            Err(e) => {
                self.stale = matches!(e, BackendError::Unavailable(_) | BackendError::Handshake(_));
                self.fail(&e);
                false
            }
        }
    }
}
