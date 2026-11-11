//! An effect given back from inside an effect or a computed (a `Drop` that runs there) does not hang on the graph lock,
//! and is gone once the lock can be taken.
//!
//! Pattern: none (tests). One test in this file on purpose: if `dispose_effect_later` took the lock directly, the
//! thread that hangs would hold the graph lock for good.
#![allow(clippy::pedantic)]

use std::sync::Mutex;
use std::sync::mpsc::channel;
use std::time::Duration;

use blinc_core::reactive::{EffectId, computed, effect, global_graph, signal};
use fs_components_primitives::{collect_field_data, dispose_effect_later};

/// What a message list's watcher is: an effect id given back when its owner is dropped.
struct Owner(EffectId);

impl Drop for Owner {
    fn drop(&mut self) {
        dispose_effect_later(self.0);
    }
}

#[test]
fn an_effect_given_back_inside_an_effect_or_a_computed_does_not_hang() {
    let effects = || global_graph().lock().unwrap().stats().effect_count;
    let base = effects();
    let watched = [effect(|_| {}), effect(|_| {})];
    assert_eq!(effects(), base + 2);
    let in_effect = Mutex::new(Some(Owner(watched[0].id())));
    let in_computed = Mutex::new(Some(Owner(watched[1].id())));
    let (done, finished) = channel();
    std::thread::spawn(move || {
        let trigger = signal(0_u8);
        // runs when made, with the graph locked, and drops its owner inside the run
        let runner = effect(move |graph| {
            let _ = graph.get(trigger);
            drop(in_effect.lock().unwrap().take());
        });
        let c = computed(move |_| {
            drop(in_computed.lock().unwrap().take());
            1_u8
        });
        let _ = c.get();
        let _ = done.send(runner.id());
    });
    let runner = finished
        .recv_timeout(Duration::from_secs(20))
        .expect("giving an effect back inside an effect hangs on the graph lock");
    // what could not go inside goes at the next call that can take the lock
    collect_field_data();
    blinc_core::reactive::dispose_host_effect(runner);
    assert_eq!(effects(), base, "both effects were given back");
}
