//! The frame of the file window: tabs, the bar of the way, the command bar, the sidebar groups.
//!
//! Tests: widget tests over a source in memory (what the real server shows is in `fs-files`).

#[path = "files_common/mod.rs"]
mod files_common;

use std::sync::Arc;

use files_common::Shelf;
use files_common::harness::Harness;
use fs_components_workbenches::file_manager::{Ask, FileLabels, FileManagerWidget};

fn window(shelf: Shelf, width: f32) -> Harness {
    let w = FileManagerWidget::new(Arc::new(shelf), FileLabels::default());
    Harness::new(w, width, 700.0)
}

#[test]
fn the_frame_shows_what_is_there_and_what_is_not_built_yet() {
    let mut h = window(Shelf::new().folder("d", None).file("a"), 1300.0);
    for what in [
        "tab-0",
        "new-tab",
        "go-back",
        "go-forward",
        "go-up",
        "reload",
        "new",
        "act-cut",
        "act-copy",
        "act-paste",
        "act-rename",
        "act-share",
        "act-delete",
        "view-list",
        "view-grid",
        "details",
        "sort",
    ] {
        assert!(h.has(what), "{what}");
    }
    // the new menu is shut until it is asked for; the order of the list is not a row of words any more
    assert!(!h.has("new-folder") && !h.has("sort-size"));
    h.click("new");
    assert!(h.has("new-folder") && h.has("upload"));
    // what is not built (cut, copy, paste, share, a second tab) does nothing, and does not fail
    for what in ["act-cut", "act-copy", "act-paste", "act-share", "new-tab"] {
        h.click(what);
    }
    assert_eq!(h.widget.context().selected, None);
    // rename and delete wait for something to act on
    h.click("act-rename");
    assert!(h.widget.with_state(|s| *s.ask() == Ask::Nothing));
}

#[test]
fn a_group_of_the_sidebar_folds_away_and_opens_again() {
    let mut h = window(Shelf::new().file("a"), 1300.0);
    assert!(h.has_id("fs-files-place-top") && h.has_id("fs-files-place-trash"));
    h.press_at(h.centre_of_id("fs-files-group-network"));
    h.rebuild();
    assert!(h.widget.with_state(|s| s.group_folded("network")), "folded");
    assert!(h.has_id("fs-files-place-trash"), "the others stay");
    h.press_at(h.centre_of_id("fs-files-group-network"));
    h.rebuild();
    assert!(
        h.widget.with_state(|s| !s.group_folded("network")),
        "open again"
    );
}

#[test]
fn the_detail_switch_gives_the_room_back_to_the_list() {
    let mut h = window(Shelf::new().file("a"), 1300.0);
    h.click("a");
    assert!(h.widget.with_state(|s| s.details_on()));
    h.click("details");
    assert!(!h.widget.with_state(|s| s.details_on()));
    h.click("details");
    assert!(h.widget.with_state(|s| s.details_on()));
}

#[test]
fn the_pinned_folders_are_the_ones_of_the_storage_that_the_host_knows_by_name() {
    use fs_components_widgets::PinnedKind;
    let shelf = Shelf::new()
        .folder("p", None)
        .folder("m", None)
        .folder("x", None);
    let w = FileManagerWidget::new(Arc::new(shelf), FileLabels::default());
    w.pin_by_names(vec![
        (PinnedKind::Pictures, "p".into()),
        (PinnedKind::Music, "m".into()),
        (PinnedKind::Videos, "no-such-folder".into()),
    ]);
    let h = Harness::new(w, 1300.0, 700.0);
    let pinned = h.widget.with_state(|s| {
        s.pinned()
            .into_iter()
            .map(|(k, f)| (k, f.name))
            .collect::<Vec<_>>()
    });
    // a kind with no folder is not shown: nothing is made up
    assert_eq!(
        pinned,
        [
            (PinnedKind::Pictures, "p".to_string()),
            (PinnedKind::Music, "m".to_string())
        ]
    );
    assert!(h.has_id("fs-files-pinned-p") && h.has_id("fs-files-pinned-m"));
    assert!(!h.has_id("fs-files-pinned-x") && !h.has_id("fs-files-pinned-no-such-folder"));
    // one harness at a time (it holds a lock of the whole process)
    drop(h);
    // counter test: with no names given nothing is pinned and the group is not there
    let none = Harness::new(
        FileManagerWidget::new(
            Arc::new(Shelf::new().folder("p", None)),
            FileLabels::default(),
        ),
        1300.0,
        700.0,
    );
    assert!(!none.has_id("fs-files-group-pinned") && !none.has_id("fs-files-pinned-p"));
}
