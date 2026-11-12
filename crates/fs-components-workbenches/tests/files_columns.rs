//! The columns of the file list as a person arranges them: hide, show, move, widen, and keep it over a restart.
//!
//! Tests: widget tests over a source in memory; the counter test shows that with no store nothing is kept.

#[path = "files_common/mod.rs"]
mod files_common;

use std::sync::Arc;

use files_common::Shelf;
use files_common::harness::Harness;
use fs_components_widgets::{FileStore, LayoutStore, NoStore};
use fs_components_workbenches::file_manager::{FileLabels, FileManagerWidget, FilesState, Variant};

fn widget(store: Arc<dyn LayoutStore>) -> FileManagerWidget {
    let w = FileManagerWidget::new(
        Arc::new(Shelf::new().file("a").file("b")),
        FileLabels::default(),
    );
    w.keep_columns_in(store, "files/list");
    w
}

fn dir(tag: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("fs-files-columns-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

#[test]
fn a_hidden_column_has_no_heading_and_no_cell_and_the_name_cannot_be_hidden() {
    let w = widget(Arc::new(NoStore));
    let mut h = Harness::new(w, 1300.0, 700.0);
    assert!(h.has("a"));
    assert!(h.has("head-size") && h.has("head-type") && h.has("head-name"));
    h.widget.with_state(|s| {
        assert!(s.columns().is_shown("size"));
        let mut next = s.columns().clone();
        assert!(next.toggle("size") && next.toggle("type"));
        assert!(!next.toggle("name"), "the name stays");
        s.set_columns(next);
    });
    h.rebuild();
    assert_eq!(h.widget.with_state(|s| s.columns().shown().len()), 2);
    // the headings of the hidden columns are gone, the others are there, and the rows still stand
    assert!(!h.has("head-size") && !h.has("head-type"));
    assert!(h.has("head-name") && h.has("head-modified"));
    assert!(h.has("a") && h.has("b"));
    // shown again
    h.widget.with_state(|s| {
        let mut next = s.columns().clone();
        next.toggle("size");
        s.set_columns(next);
    });
    h.rebuild();
    assert!(h.has("head-size"));
}

#[test]
fn order_hidden_and_width_come_back_after_a_restart() {
    let folder = dir("restart");
    let first = widget(Arc::new(FileStore::new(&folder)));
    first.with_state(|s| {
        let mut next = s.columns().clone();
        next.toggle("modified");
        next.move_by("type", -2);
        next.set_width("name", 260.0);
        s.set_columns(next);
    });
    let was = first.with_state(|s| s.columns().to_text());
    drop(first);
    // the restart: a new widget, a new store over the same folder
    let second = widget(Arc::new(FileStore::new(&folder)));
    assert_eq!(second.with_state(|s| s.columns().to_text()), was);
    assert!(second.with_state(
        |s| !s.columns().is_shown("modified") && s.columns().width_of("name") == Some(260.0)
    ));
    let _ = std::fs::remove_dir_all(folder);
}

#[test]
fn counter_test_with_no_store_the_choice_is_gone_after_a_restart() {
    let first = widget(Arc::new(NoStore));
    first.with_state(|s| {
        let mut next = s.columns().clone();
        next.toggle("size");
        s.set_columns(next);
    });
    assert!(!first.with_state(|s| s.columns().is_shown("size")));
    let second = widget(Arc::new(NoStore));
    assert!(
        second.with_state(|s| s.columns().is_shown("size")),
        "kept nothing: back to the default"
    );
    let _: Option<Variant> = None;
    let _: Option<&FilesState> = None;
}
