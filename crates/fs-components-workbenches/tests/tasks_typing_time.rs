//! How long one key takes in the task form: into the one-line title, into the many-line notes (empty
//! and with a long text already in it), and a full rebuild for comparison.
//! Run: `cargo test --release --test tasks_typing_time -- --ignored --nocapture`.
//!
//! Tests: a measurement, not a check; it prints numbers and asserts nothing about speed.

#[path = "tasks_common/mod.rs"]
mod tasks_common;

use std::sync::Arc;
use std::time::{Duration, Instant};

use fs_components_workbenches::tasks::{TaskLabels, TasksWidget};
use tasks_common::Shelf;
use tasks_common::harness::Harness;

fn median(mut v: Vec<Duration>) -> Duration {
    v.sort();
    v[v.len() / 2]
}

fn keys(h: &mut Harness, field: &str, text: &str, n: usize) -> (Duration, Duration) {
    h.press(field);
    assert!(h.type_text(text), "{field} has the focus");
    let (mut key, mut frame) = (vec![], vec![]);
    for _ in 0..n {
        let t = Instant::now();
        h.type_text("x");
        key.push(t.elapsed());
        let t = Instant::now();
        h.frame();
        frame.push(t.elapsed());
    }
    (median(key), median(frame))
}

#[test]
#[ignore = "a measurement"]
fn one_key_takes_this_long() {
    let shelf = Arc::new(Shelf::with(&["a"]));
    let mut h = Harness::new(
        TasksWidget::new(shelf, TaskLabels::default()),
        1200.0,
        800.0,
    );
    h.click("edit");
    let (k, f) = keys(&mut h, "field-title", "", 40);
    eprintln!("title: key {k:?}, frame {f:?}");
    let (k, f) = keys(&mut h, "field-notes", "", 40);
    eprintln!("notes (empty): key {k:?}, frame {f:?}");
    let long = "A line of a longer text for the notes field. ".repeat(60);
    let (k, f) = keys(&mut h, "field-notes", &long, 40);
    eprintln!("notes (2.7 KB): key {k:?}, frame {f:?}");
    let rebuilds: Vec<Duration> = (0..20)
        .map(|_| {
            let t = Instant::now();
            h.rebuild();
            t.elapsed()
        })
        .collect();
    eprintln!("full rebuild {:?}", median(rebuilds));
}
