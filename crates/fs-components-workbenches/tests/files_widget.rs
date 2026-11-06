//! The workbench driven through Blinc's router and the keys a window sends. No window is opened.
//!
//! Pattern: input tests. A hang here is a deadlock: run with a timeout.

#[path = "files_common/mod.rs"]
mod files_common;

use std::sync::{Arc, Mutex};

use files_common::harness::{Harness, PickerHarness};
use files_common::{Shelf, widget};
use fs_components_workbenches::file_manager::{
    Ask, FileLabels, FilePicker, FileView, PendingUpload, PickedFile,
};

const WIDTHS: [f32; 8] = [1300.0, 900.0, 720.0, 560.0, 400.0, 300.0, 200.0, 160.0];

/// Press the point twice; only the first press may mean anything, whatever is drawn by then. Then the
/// answer stands elsewhere, inside the window, and pressing it does the thing.
fn second_act_is_elsewhere(h: &mut Harness, width: f32, first: &str, redraw: bool) {
    second_act_with(h, width, first, redraw, |_| {});
}

fn second_act_with(
    h: &mut Harness,
    width: f32,
    first: &str,
    redraw: bool,
    again: impl Fn(&mut Harness),
) {
    let point = h.centre(first);
    h.press_at(point);
    if redraw {
        h.rebuild();
    }
    h.press_at(point);
    h.rebuild();
    if !h.has("really") {
        again(h);
        h.click(first);
    }
    let (left, top, right, bottom) = h.edges("really");
    assert!(
        right <= width + 0.5 && left >= -0.5,
        "{width}: {left}..{right}"
    );
    assert!(bottom <= 700.0 + 0.5, "{width}: {top}..{bottom}");
    let under = point.0 >= left && point.0 <= right && point.1 >= top && point.1 <= bottom;
    assert!(!under, "{width}: the first press lands on it");
}

#[test]
fn delete_at_every_width_with_and_without_a_redraw_never_acts_on_a_double_press() {
    for width in WIDTHS {
        for redraw in [false, true] {
            let (shelf, w) = widget(Shelf::new().file("a").file("b"));
            let mut h = Harness::new(w, width, 700.0);
            h.click("a");
            second_act_is_elsewhere(&mut h, width, "delete", redraw);
            assert_eq!(shelf.writes(), 0, "{width} {redraw}");
            assert!(!shelf.thing("a").unwrap().trashed);
            h.click("really");
            assert!(
                shelf.thing("a").unwrap().trashed,
                "{width}: the press reached it"
            );
        }
    }
}

#[test]
fn purge_and_empty_trash_are_two_acts_at_every_width_and_say_how_many() {
    for width in WIDTHS {
        for redraw in [false, true] {
            let (shelf, w) = widget(Shelf::new().file("a").file("b").file("c"));
            for key in ["a", "b", "c"] {
                shelf.change(key, |t| t.trashed = true);
            }
            w.with_state(|s| s.open_trash());
            let mut h = Harness::new(w, width, 700.0);
            h.click("a");
            second_act_is_elsewhere(&mut h, width, "delete", redraw);
            assert!(shelf.thing("a").is_some(), "{width} {redraw}");
            h.click("really");
            assert!(shelf.thing("a").is_none(), "{width}");
            if width < 560.0 {
                h.click("tools");
            }
            second_act_with(&mut h, width, "empty-trash", redraw, |h| {
                // At narrow widths the question took the tools' room, and the second press went on
                // to whatever was under it. That can only be something harmless.
                h.widget.with_state(|s| {
                    s.keep();
                    s.open_trash();
                    if !s.tools_open() {
                        s.toggle_tools();
                    }
                });
                h.rebuild();
            });
            assert_eq!(shelf.writes(), 1, "{width}: only the purge");
            let words = h.widget.with_state(|s| s.confirm_words()).unwrap().0;
            assert!(words.contains('2'), "{words}");
            h.click("really");
            assert!(shelf.thing("b").is_none() && shelf.thing("c").is_none());
        }
    }
}

#[test]
fn cancel_with_something_typed_is_two_acts_and_typing_is_real_key_events() {
    for width in [1300.0, 560.0, 200.0] {
        for redraw in [false, true] {
            let (shelf, w) = widget(Shelf::new().file("a"));
            let mut h = Harness::new(w, width, 700.0);
            h.click("a");
            h.click("rename");
            h.press("line");
            assert!(h.type_text("xy"), "the field has the focus after a click");
            let value = h.widget.with_state(|s| match s.ask() {
                Ask::Input(i) => i.value.clone(),
                _ => panic!("no input"),
            });
            assert!(value.contains("xy"), "{value}");
            let point = h.centre("cancel");
            h.press_at(point);
            if redraw {
                h.rebuild();
            }
            h.press_at(point);
            h.rebuild();
            assert!(
                h.widget.with_state(|s| matches!(s.ask(), Ask::Input(_))),
                "{width} {redraw}: a double press keeps what was typed"
            );
            if !h.has("discard") {
                h.click("cancel");
            }
            let (left, top, right, bottom) = h.edges("discard");
            let under = point.0 >= left && point.0 <= right && point.1 >= top && point.1 <= bottom;
            assert!(!under, "{width}: the first press lands on it");
            h.click("discard");
            assert!(h.widget.with_state(|s| *s.ask() == Ask::Nothing));
            assert_eq!(shelf.writes(), 0);
        }
    }
}

#[test]
fn a_taken_name_asks_and_keep_both_is_reachable_at_every_width() {
    for width in WIDTHS {
        let (shelf, w) = widget(Shelf::new().file("a").file("b"));
        let mut h = Harness::new(w, width, 700.0);
        h.click("a");
        h.click("rename");
        h.press("line");
        h.type_text("b.txt\u{0}");
        // The click puts the cursor at the start: set the whole line the way typing ends up.
        h.widget.with_state(|s| s.set_input("b.txt"));
        h.rebuild();
        h.click("apply");
        assert!(
            h.widget
                .with_state(|s| matches!(s.ask(), Ask::NameTaken(_))),
            "{width}"
        );
        assert_eq!(shelf.thing("a").unwrap().name, "a.txt", "nothing changed");
        assert!(!h.has("replace"), "there is no replace");
        let (left, _, right, bottom) = h.edges("keep-both");
        assert!(
            left >= -0.5 && right <= width + 0.5 && bottom <= 700.5,
            "{width}"
        );
        h.click("keep-both");
        assert_eq!(shelf.thing("a").unwrap().name, "b.txt (2)", "{width}");
    }
}

#[test]
fn the_toolbar_is_inside_the_window_at_every_width() {
    for width in WIDTHS {
        let (_, w) = widget(Shelf::new().file("a"));
        let mut h = Harness::new(w, width, 700.0);
        if width < 560.0 {
            h.click("tools");
        }
        for what in [
            "filter",
            "view-list",
            "view-grid",
            "trash",
            "new-folder",
            "upload",
            "rescan",
            "transfers",
            "sort-size",
        ] {
            let (left, top, right, bottom) = h.edges(what);
            assert!(
                left >= -0.5 && right <= width + 0.5,
                "{width} {what}: {left}..{right}"
            );
            assert!(bottom <= 700.0, "{width} {what}: {top}..{bottom}");
            let (_, crumbs, _, _) = h.edges("crumb-0");
            assert!(bottom <= crumbs, "{width} {what}: the bar is high enough");
        }
    }
}

#[test]
fn a_double_press_on_a_folder_goes_one_level_and_nothing_opens_by_itself() {
    let (_, w) = widget(
        Shelf::new()
            .folder("d", None)
            .folder("e", Some("d"))
            .file_in("x", Some("e")),
    );
    assert_eq!(w.context().selected, None, "nothing selected at the start");
    let mut h = Harness::new(w, 1300.0, 700.0);
    let point = h.centre("d");
    h.press_at(point);
    h.rebuild();
    h.press_at(point);
    h.rebuild();
    assert_eq!(h.widget.context().folder.as_deref(), Some("d"));
    assert_eq!(h.widget.context().selected, None);
}

#[test]
fn actions_are_reached_by_tab_and_enter() {
    let (shelf, w) = widget(Shelf::new().file("a"));
    w.with_state(|s| s.select("a"));
    let mut h = Harness::new(w, 1300.0, 700.0);
    assert!(h.tab_to("delete"));
    h.key(13);
    h.rebuild();
    assert!(h.has("really"), "Enter on Delete asks");
    assert!(h.tab_to("really"));
    h.key(13);
    h.rebuild();
    assert!(
        shelf.thing("a").unwrap().trashed,
        "Enter on the answer does it"
    );
}

#[test]
fn breadcrumb_goes_back_and_the_list_starts_at_the_top_there() {
    let (_, w) = widget(Shelf::new().folder("d", None).numbered(120));
    w.with_state(|s| s.open_folder(Some("d")));
    let mut h = Harness::new(w, 1300.0, 700.0);
    assert!(h.has("crumb-0"));
    h.click("crumb-0");
    assert_eq!(h.widget.context().folder, None);
}

#[test]
fn the_grid_is_a_view_of_the_same_rows_and_only_what_is_in_view_exists() {
    let (_, w) = widget(Shelf::new().numbered(5000));
    w.with_state(|s| {
        while s.has_more() && s.rows().len() < 5000 {
            s.load_more();
        }
        s.set_view(FileView::Grid);
    });
    let mut h = Harness::new(w, 1300.0, 700.0);
    h.frame();
    assert!(h.has("f00000"));
    assert!(!h.has("f04000"), "far tiles do not exist");
    h.click("f00001");
    assert_eq!(h.widget.context().selected.as_deref(), Some("f00001"));
}

#[test]
fn search_keeps_the_field_redraws_the_list_only_and_a_failure_keeps_the_rows() {
    let (shelf, w) = widget(Shelf::new().file("a").file("b"));
    let mut h = Harness::new(w, 1300.0, 700.0);
    h.press("search");
    assert!(
        h.type_text("a."),
        "the search field has the focus after a click"
    );
    h.frame();
    assert!(h.has("a"));
    assert!(!h.has("b"), "the live area followed");
    shelf.break_it(Some("the store is gone"));
    h.widget.with_state(|s| s.retry());
    h.rebuild();
    assert!(h.has("a"), "the rows stay");
    assert!(h.has("list-problem") && h.has("retry"));
    shelf.break_it(None);
    h.click("retry");
    assert!(!h.has("list-problem"));
}

#[test]
fn a_new_folder_is_typed_with_real_key_events_and_a_bad_name_stays_in_the_field() {
    let (shelf, w) = widget(Shelf::new().file("a"));
    let mut h = Harness::new(w, 1300.0, 700.0);
    h.click("new-folder");
    h.press("line");
    assert!(h.type_text("ab/c"), "the field has the focus after a click");
    h.rebuild();
    h.click("apply");
    assert!(h.has("input-error"), "said as such");
    assert_eq!(shelf.writes(), 0);
    let typed = h.widget.with_state(|s| match s.ask() {
        Ask::Input(input) => input.value.clone(),
        _ => panic!("no input"),
    });
    assert_eq!(typed, "ab/c", "what was typed stays");
    h.widget.with_state(|s| s.set_input("Fine name"));
    h.rebuild();
    h.click("apply");
    assert!(shelf.thing("new-Fine name").is_some());
    assert!(h.has("new-Fine name"), "the new row is in the list");
}

#[test]
fn a_press_in_the_tree_goes_to_that_folder_and_it_opens() {
    let (_, w) = widget(
        Shelf::new()
            .folder("alpha", None)
            .folder("beta", None)
            .folder("inner", Some("beta")),
    );
    let mut h = Harness::new(w, 1300.0, 700.0);
    let (left, top, right, _) = h.edges("tree");
    let x = f32::midpoint(left, right);
    // The rows are 28 high: the top, alpha, beta.
    h.press_at((x, top + 14.0 + 28.25 * 2.0));
    h.rebuild();
    assert_eq!(h.widget.context().folder.as_deref(), Some("beta"));
    let opened = h
        .widget
        .with_state(|s| s.tree_nodes()[0].children[1].children.len());
    assert_eq!(opened, 1, "its folders show");
    h.press_at((x, top + 14.0));
    h.rebuild();
    assert_eq!(h.widget.context().folder, None, "the top");
}

#[test]
fn keys_reach_the_grid_and_the_list_from_tab() {
    for view in [FileView::Grid, FileView::List] {
        let (_, w) = widget(Shelf::new().numbered(30));
        w.with_state(|s| s.set_view(view));
        let mut h = Harness::new(w, 1300.0, 700.0);
        h.frame();
        let name = h.widget.element_name();
        assert!(h.has("f00000"), "{name}");
        assert!(h.tab_to_list(&name), "{view:?}");
        h.key(40);
        h.rebuild();
        assert!(
            h.widget.context().selected.is_some(),
            "{view:?}: down selects"
        );
    }
}

#[test]
fn going_elsewhere_clears_the_search_field_too() {
    let (_, w) = widget(Shelf::new().folder("d", None).file("a").file("b"));
    let mut h = Harness::new(w, 1300.0, 700.0);
    h.press("search");
    h.type_text("a.");
    h.frame();
    assert_eq!(h.widget.with_state(|s| s.term().to_string()), "a.");
    h.widget.with_state(|s| s.open_folder(Some("d")));
    h.rebuild();
    h.press("search");
    h.type_text("z");
    h.frame();
    assert_eq!(
        h.widget.with_state(|s| s.term().to_string()),
        "z",
        "the field was empty"
    );
    // Nothing of the old field is left holding the keyboard: keys reach the list again.
    h.widget.with_state(|s| s.search(""));
    h.widget.with_state(|s| s.open_folder(None));
    h.rebuild();
    let name = h.widget.element_name();
    assert!(h.tab_to_list(&name));
    h.key(40);
    assert!(h.widget.context().selected.is_some(), "down selects");
}

#[test]
fn the_buttons_of_the_transfers_answer_through_the_router() {
    let shelf = Shelf::new();
    shelf
        .pending
        .lock()
        .unwrap()
        .push(PendingUpload::new("u1", "big.iso", 500));
    let (shelf, w) = widget(shelf);
    w.with_state(|s| s.open_transfers());
    let mut h = Harness::new(w, 1300.0, 700.0);
    let point = h.point_of_id("fs-tl-pending-u1-dismiss");
    h.press_at(point);
    h.rebuild();
    assert!(h.has("really"), "asks");
    assert_eq!(shelf.pending.lock().unwrap().len(), 1);
    h.click("really");
    assert!(shelf.pending.lock().unwrap().is_empty());
}

#[test]
fn the_picker_gives_back_a_value_and_offers_nothing_else() {
    let shelf = Arc::new(Shelf::new().file("a").folder("d", None));
    let picker = FilePicker::new(shelf.clone(), FileLabels::default());
    let got = Arc::new(Mutex::new(None));
    let seen = got.clone();
    let picker = picker.on_pick(Arc::new(move |p: &PickedFile| {
        *seen.lock().unwrap() = Some(p.key.clone());
    }));
    let mut h = PickerHarness::new(picker, 500.0, 500.0);
    assert!(h.has("a"));
    assert!(!h.has("trash") && !h.has("new-folder") && !h.has("upload"));
    h.click("a");
    assert!(!h.has("delete") && !h.has("rename"));
    h.click("choose");
    assert_eq!(got.lock().unwrap().as_deref(), Some("a"));
    assert_eq!(shelf.writes(), 0);
}
