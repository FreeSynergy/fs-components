//! The arrow keys at the edge of the list: a key that leads nowhere does not open the folder
//! that is selected.
//!
//! Pattern: input tests through the router a window uses. A hang here is a deadlock.

#[path = "files_common/mod.rs"]
mod files_common;

use files_common::harness::Harness;
use files_common::{Shelf, widget};
use fs_components_workbenches::file_manager::FileView;

const UP: u32 = 0x26;
const DOWN: u32 = 0x28;
const HOME: u32 = 0x24;
const END: u32 = 0x23;

fn edge_keys(view: FileView) {
    let (_, w) = widget(Shelf::new().folder("d", None));
    w.with_state(|s| {
        s.set_view(view);
        s.select("d");
    });
    let mut h = Harness::new(w, 1300.0, 700.0);
    assert!(h.tab_to_list(), "the list takes the keyboard");
    for key in [DOWN, END, UP, HOME] {
        h.key(key);
        h.rebuild();
        assert_eq!(
            h.widget.context().folder,
            None,
            "{view:?} {key:#x} opened it"
        );
    }
    assert_eq!(h.widget.context().selected.as_deref(), Some("d"));
}

#[test]
fn a_key_at_the_end_of_the_list_does_not_open_the_folder_selected() {
    edge_keys(FileView::List);
}

#[test]
fn a_key_at_the_end_of_the_grid_does_not_open_the_folder_selected() {
    edge_keys(FileView::Grid);
}
