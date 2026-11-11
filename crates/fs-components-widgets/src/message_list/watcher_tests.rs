//! The watcher of a message list given back inside an effect.
//!
//! Pattern: none (tests). A `Watcher` is dropped when the last clone of a list is; that can happen inside an effect or a
//! computed, which hold the graph lock while they run.

use std::sync::Mutex;
use std::sync::mpsc::channel;
use std::time::Duration;

use blinc_core::reactive::{computed, effect, signal};

use super::handle::Watcher;

#[test]
fn the_last_clone_dropped_inside_an_effect_or_a_computed_does_not_hang() {
    let in_effect = Mutex::new(Some(Watcher(Mutex::new(Some(effect(|_| {}))))));
    let in_computed = Mutex::new(Some(Watcher(Mutex::new(Some(effect(|_| {}))))));
    let (done, finished) = channel();
    std::thread::spawn(move || {
        let trigger = signal(0_u8);
        let runner = effect(move |graph| {
            let _ = graph.get(trigger);
            drop(in_effect.lock().unwrap().take());
        });
        let c = computed(move |_| {
            drop(in_computed.lock().unwrap().take());
            1_u8
        });
        let _ = c.get();
        let _ = done.send(runner);
    });
    assert!(
        finished.recv_timeout(Duration::from_secs(20)).is_ok(),
        "dropping the watcher inside an effect hangs on the graph lock"
    );
}
