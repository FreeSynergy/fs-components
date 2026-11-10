//! The worker that draws a page again when the messages change.
//!
//! Pattern: Observer (the messages tell it by a signal) with a worker thread as the receiving end. Messages change while
//! the session is locked (a transition), and a draw reads the session, so the draw must not run on the caller's thread:
//! it is asked by a signal and done here, which waits for the lock. Signals that arrive while a draw runs are folded
//! into one more draw. It draws once more when `expiry` has passed after the last signal (a success runs out; messages
//! have no timer of their own).
//!
//! One thread per surface. It ends when the last signal sender is dropped, i.e. when the session (which holds the
//! listener) is dropped; Blinc may keep the page closures, and with them the session, for the life of the window, so in
//! a running program it lives as long as the window.

use std::sync::mpsc::{RecvTimeoutError, channel};
use std::sync::{Mutex, PoisonError};
use std::time::Duration;

/// Start the worker; the returned function is the signal to hand to `Messages::subscribe`.
pub(super) fn redraw_worker(
    draw: impl Fn() + Send + 'static,
    expiry: Duration,
) -> impl Fn() + Send + Sync + 'static {
    let (tx, rx) = channel::<()>();
    std::thread::spawn(move || {
        let mut armed = false;
        loop {
            match rx.recv_timeout(expiry) {
                Ok(()) => {
                    while rx.try_recv().is_ok() {} // a burst of signals is one draw
                    draw();
                    armed = true;
                }
                Err(RecvTimeoutError::Timeout) => {
                    if std::mem::take(&mut armed) {
                        draw();
                    }
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    });
    let tx = Mutex::new(tx);
    move || {
        let _ = tx.lock().unwrap_or_else(PoisonError::into_inner).send(());
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use super::redraw_worker;

    #[test]
    fn a_burst_of_signals_is_a_few_draws_and_a_quiet_time_draws_once_more() {
        let draws = Arc::new(AtomicUsize::new(0));
        let d = Arc::clone(&draws);
        let signal = redraw_worker(
            move || {
                d.fetch_add(1, Ordering::SeqCst);
                std::thread::sleep(Duration::from_millis(5)); // a draw takes time
            },
            Duration::from_millis(300),
        );
        for _ in 0..200 {
            signal();
        }
        std::thread::sleep(Duration::from_millis(150));
        let burst = draws.load(Ordering::SeqCst);
        assert!(burst >= 1, "it draws");
        assert!(burst < 30, "200 signals made {burst} draws");
        std::thread::sleep(Duration::from_millis(500));
        assert_eq!(
            draws.load(Ordering::SeqCst),
            burst + 1,
            "once more when the quiet time has passed"
        );
    }
}
