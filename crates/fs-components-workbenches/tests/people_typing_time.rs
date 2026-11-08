//! How long one key takes in the form: the key into the field, the frame after it, and, for
//! comparison, a full rebuild of the tree. Run: `cargo test --release --test people_typing_time -- --ignored --nocapture`.
//!
//! Tests: a measurement, not a check; it prints numbers and asserts nothing about speed.

#[path = "people_common/mod.rs"]
mod people_common;

use std::sync::Arc;
use std::time::{Duration, Instant};

use fs_components_workbenches::people::{PeopleLabels, PeopleWidget};
use people_common::{Book, harness::Harness};

fn median(mut v: Vec<Duration>) -> Duration {
    v.sort();
    v[v.len() / 2]
}

#[test]
#[ignore = "a measurement"]
fn one_key_and_one_rebuild_take_this_long() {
    let book = Arc::new(Book::with(&["ada", "bob"]));
    let widget = PeopleWidget::new(book, PeopleLabels::default());
    let mut h = Harness::new(widget, 1200.0, 800.0);
    h.click("edit");
    h.press("field-name");
    let (mut keys, mut frames) = (vec![], vec![]);
    for _ in 0..60 {
        let t = Instant::now();
        h.type_text("x");
        keys.push(t.elapsed());
        let t = Instant::now();
        h.frame();
        frames.push(t.elapsed());
    }
    let rebuilds: Vec<Duration> = (0..20)
        .map(|_| {
            let t = Instant::now();
            h.rebuild();
            t.elapsed()
        })
        .collect();
    eprintln!(
        "key {:?}, frame after it {:?}, full rebuild {:?}",
        median(keys),
        median(frames),
        median(rebuilds)
    );
}
