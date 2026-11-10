//! The worker that draws a page again when the messages change.
//!
//! Pattern: Observer (the messages tell it by a signal) with a worker thread as the receiving end. Messages change
//! while the session is locked (a transition), and a draw reads the session, so the draw must not run on the caller's
//! thread: it is asked by a signal and done here, which waits for the lock. Signals that arrive while a draw runs are
//! folded into one more draw. It draws once more when `expiry` has passed after the last signal (a success runs out;
//! messages have no timer of their own).
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
            Duration::from_secs(2),
        );
        for _ in 0..200 {
            signal();
        }
        // wait (with a limit) until it has drawn, then until it is quiet: no fixed pauses that load could upset
        let limit = std::time::Instant::now() + Duration::from_secs(10);
        while draws.load(Ordering::SeqCst) == 0 && std::time::Instant::now() < limit {
            std::thread::sleep(Duration::from_millis(5));
        }
        let mut last = draws.load(Ordering::SeqCst);
        loop {
            std::thread::sleep(Duration::from_millis(60));
            let now = draws.load(Ordering::SeqCst);
            if now == last && now > 0 {
                break;
            }
            last = now;
            assert!(std::time::Instant::now() < limit, "never quiet");
        }
        assert!(last < 30, "200 signals made {last} draws");
        // the quiet time (2 s) after the last signal brings exactly one more
        let limit = std::time::Instant::now() + Duration::from_secs(10);
        while draws.load(Ordering::SeqCst) == last && std::time::Instant::now() < limit {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(draws.load(Ordering::SeqCst), last + 1);
        // and then nothing more: wait longer than the quiet time
        std::thread::sleep(Duration::from_millis(2500));
        assert_eq!(
            draws.load(Ordering::SeqCst),
            last + 1,
            "it draws again without a signal"
        );
    }
}
