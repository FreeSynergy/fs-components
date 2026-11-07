//! How long a build takes with very many rows. Not a pass or fail: run by hand with
//! `cargo test --release -p fs-components-workbenches --test files_measure -- --ignored --nocapture`.
//!
//! Pattern: measurement over a source in memory, through the harness.

#[path = "files_common/mod.rs"]
mod files_common;

use std::time::Instant;

use files_common::harness::Harness;
use files_common::{Shelf, widget};
use fs_components_workbenches::file_manager::FileView;

#[test]
#[ignore = "measures"]
fn many_rows_loaded() {
    for count in [5_000, 50_000] {
        for view in [FileView::List, FileView::Grid] {
            let (_, w) = widget(Shelf::new().numbered(count));
            w.with_state(|s| {
                *s = s.clone();
                while s.has_more() {
                    s.load_more();
                }
                s.set_view(view);
            });
            let mut h = Harness::new(w, 1300.0, 800.0);
            let began = Instant::now();
            for _ in 0..3 {
                h.rebuild();
            }
            eprintln!(
                "{count} rows {view:?}: {:.0} ms a build",
                began.elapsed().as_secs_f64() * 1000.0 / 3.0
            );
        }
    }
}
