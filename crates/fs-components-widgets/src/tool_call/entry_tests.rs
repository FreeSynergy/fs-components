//! The watch of a drawn question given back inside an effect.
//!
//! Pattern: none (tests). A `Watch` is given back when its entry goes; that can happen inside an effect or a computed,
//! which hold the graph lock while they run.

use std::sync::Mutex;
use std::sync::mpsc::channel;
use std::time::Duration;

use blinc_core::reactive::{computed, effect, signal};

use super::entry::Watch;

fn a_watch() -> Watch {
    Watch {
        effect: effect(|_| {}).id(),
        offset: signal(0.0_f32).id(),
    }
}

#[test]
fn a_watch_given_back_inside_an_effect_or_a_computed_does_not_hang() {
    let in_effect = Mutex::new(Some(a_watch()));
    let in_computed = Mutex::new(Some(a_watch()));
    let (done, finished) = channel();
    std::thread::spawn(move || {
        let trigger = signal(0_u8);
        let runner = effect(move |graph| {
            let _ = graph.get(trigger);
            if let Some(watch) = in_effect.lock().unwrap().take() {
                watch.dispose();
            }
        });
        let c = computed(move |_| {
            if let Some(watch) = in_computed.lock().unwrap().take() {
                watch.dispose();
            }
            1_u8
        });
        let _ = c.get();
        let _ = done.send(runner);
    });
    assert!(
        finished.recv_timeout(Duration::from_secs(20)).is_ok(),
        "giving a watch back inside an effect hangs on the graph lock"
    );
}
