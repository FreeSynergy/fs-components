//! The action of "Save" and "Yes": a command goes to the program on a worker.
//!
//! Pattern: Command object run off the interface thread (the halves of `Session::submit_begin` / `submit_finish`).

use std::sync::{Arc, Mutex, PoisonError};

use super::chrome::Action;
use super::surface::{AdminSurface, lock};

impl AdminSurface {
    /// The action of "Save" and "Yes": the command goes to the program on a worker, with no lock held and not on the
    /// interface thread, so the window stays usable however long the server takes. The page shows "running" meanwhile
    /// (`message.running`); a second press while it runs does nothing. The answer is put in under the lock and the page
    /// is drawn again from the worker (the same way as "Try again").
    pub(super) fn submit_action(&self) -> Action {
        let (session, live) = (Arc::clone(&self.env.session), self.live);
        Arc::new(move || {
            let Some(job) = lock(&session).submit_begin() else {
                live.refresh_now(); // nothing to send (the form marks its findings), or one command is on its way
                return;
            };
            live.refresh_now(); // shows "running"
            let slot = Arc::new(Mutex::new(Some(job)));
            let (later, taken) = (Arc::clone(&session), Arc::clone(&slot));
            let spawned = std::thread::Builder::new()
                .name("fs-admin-command".into())
                .spawn(move || {
                    let job = taken.lock().unwrap_or_else(PoisonError::into_inner).take();
                    if let Some(job) = job {
                        let outcome = job.run();
                        lock(&later).submit_finish(outcome);
                        live.refresh();
                    }
                });
            if spawned.is_err() {
                // No thread to be had: the failure is told, never silence.
                let job = slot.lock().unwrap_or_else(PoisonError::into_inner).take();
                if let Some(job) = job {
                    let failed = crate::SubmitOutcome::failed(
                        &job,
                        crate::BackendError::Unavailable("no thread to send the command".into()),
                    );
                    lock(&session).submit_finish(failed);
                }
                live.refresh_now();
            }
        })
    }
}
