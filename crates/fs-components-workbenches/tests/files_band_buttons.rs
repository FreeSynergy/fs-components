//! The buttons of the files band: every question has its buttons inside their frame and the window, even
//! where the words are long and the window is narrow (160 px). The test measures the button itself.
//!
//! Pattern: Picture and input tests. No window is opened. A hang here is a deadlock: run with a timeout.

#[path = "files_common/mod.rs"]
mod files_common;

use files_common::harness::Harness;
use files_common::{Shelf, widget};
use fs_components_workbenches::file_manager::{CarryKind, FileLabels, FilesState};

const WIDTHS: [f32; 3] = [300.0, 200.0, 160.0];

/// What puts the band into one state, and the buttons that must then be there.
type Scene = (&'static str, fn(&mut FilesState), &'static [&'static str]);

fn shelf() -> Shelf {
    let shelf = Shelf::new().file("a").file("t").file("m").folder("f", None);
    shelf.change("t", |t| t.trashed = true);
    shelf.change("m", |t| t.marked = true);
    shelf
}

const SCENES: [Scene; 12] = [
    (
        "trash",
        |s| {
            s.select("a");
            s.ask_delete();
        },
        &["keep-it", "really"],
    ),
    (
        "delete folder",
        |s| {
            s.select("f");
            s.ask_delete();
        },
        &["keep-it", "really"],
    ),
    (
        "delete forever",
        |s| {
            s.open_trash();
            s.select("t");
            s.ask_delete();
        },
        &["keep-it", "really"],
    ),
    (
        "empty trash",
        |s| {
            s.open_trash();
            s.ask_empty_trash();
        },
        &["keep-it", "really"],
    ),
    (
        "write a marked file",
        |s| {
            s.select("m");
            s.begin_download();
            s.set_input("/there/m.txt");
            s.submit_input();
        },
        &["keep-it", "really"],
    ),
    (
        "taken name",
        |s| {
            s.select("a");
            s.begin_rename();
            s.set_input("m.txt");
            s.submit_input();
        },
        &["back", "keep-both"],
    ),
    (
        "carry",
        |s| {
            s.select("a");
            s.carry(CarryKind::Move);
            s.open_folder(Some("f"));
        },
        &["stop-carrying", "put-down"],
    ),
    (
        "copy",
        |s| {
            s.select("a");
            s.carry(CarryKind::Copy);
            s.open_folder(Some("f"));
        },
        &["stop-carrying", "put-down"],
    ),
    (
        "input",
        |s| {
            s.select("a");
            s.begin_rename();
        },
        &["cancel", "apply"],
    ),
    (
        "input with a change",
        |s| {
            s.select("a");
            s.begin_rename();
            s.set_input("x");
        },
        &["cancel", "apply"],
    ),
    (
        "discard asked",
        |s| {
            s.select("a");
            s.begin_rename();
            s.set_input("x");
            s.cancel_input();
        },
        &["keep-typing", "discard"],
    ),
    (
        "new folder",
        FilesState::begin_new_folder,
        &["cancel", "apply"],
    ),
];

fn inside(h: &Harness, what: &str, scene: &str, width: f32) {
    assert!(h.has(what), "{width} {scene}: there is no {what}");
    let (fl, _, fr, _) = h.edges(what);
    let (bl, bt, br, bb) = h.button_edges(what);
    assert!(
        bt >= -0.5 && bb <= 700.5,
        "{width} {scene} {what}: button {bt}..{bb} is outside the window height"
    );
    assert!(
        bl >= fl - 0.5 && br <= fr + 0.5,
        "{width} {scene} {what}: button {bl}..{br} is outside its frame {fl}..{fr}"
    );
    assert!(
        bl >= -0.5 && br <= width + 0.5,
        "{width} {scene} {what}: button {bl}..{br} is outside the window"
    );
}

#[test]
fn every_question_of_the_band_has_its_buttons_inside_the_window_when_narrow() {
    for width in WIDTHS {
        for (scene, put, buttons) in SCENES {
            let (_, w) = widget(shelf());
            let mut h = Harness::new(w, width, 700.0);
            h.widget.with_state(put);
            h.rebuild();
            for what in buttons {
                inside(&h, what, scene, width);
            }
            // The band is a box as high as its estimate and clips what is taller; the double-press rule (the
            // answer is never under the point of the first press) stands on where the buttons are. So every button
            // must lie inside the box: a too small estimate turns this red.
            let (_, top, _, bottom) = h.edges("band");
            for what in buttons {
                let (_, bt, _, bb) = h.button_edges(what);
                assert!(
                    bt >= top - 0.5 && bb <= bottom + 0.5,
                    "{width} {scene} {what}: button {bt}..{bb}, the band {top}..{bottom}"
                );
            }
        }
    }
}

#[test]
fn a_long_answer_wraps_into_lines_instead_of_widening_the_button() {
    let (_, w) = widget(shelf());
    let mut h = Harness::new(w, 160.0, 700.0);
    h.widget.with_state(SCENES[4].1);
    h.rebuild();
    let (_, top, _, bottom) = h.button_edges("really");
    assert!(
        bottom - top > 30.0,
        "{} px high: it did not wrap",
        bottom - top
    );
    assert!(
        FileLabels::default().download_marked_really.chars().count() * 7 > 140,
        "the answer is longer than the room"
    );
}
