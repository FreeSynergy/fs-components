//! Asking the window to close.
//!
//! Pattern: none (a slot for one callback). The shell cannot close its window itself: `launch` hands it the window's
//! close callback, and a menu handler (Quit) calls [`AppWidget::quit`]. The event loop then ends and everything is
//! dropped in order, instead of the program ending with `std::process::exit`.

use std::sync::{Arc, Mutex, PoisonError};

use crate::widget::AppWidget;

type CloseFn = Arc<dyn Fn() + Send + Sync>;

/// Where the close callback of the window is kept; clones share it.
#[derive(Clone, Default)]
pub(crate) struct QuitSlot(Arc<Mutex<Option<CloseFn>>>);

impl AppWidget {
    /// Set what closes the window (`launch` does; a test sets its own).
    pub fn set_quit(&self, close: impl Fn() + Send + Sync + 'static) {
        *self.quit.0.lock().unwrap_or_else(PoisonError::into_inner) = Some(Arc::new(close));
    }

    /// Ask the window to close. Returns `false` if no window is open to close (nothing was asked).
    pub fn quit(&self) -> bool {
        let close = self
            .quit
            .0
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone();
        close.is_some_and(|close| {
            close();
            true
        })
    }
}
