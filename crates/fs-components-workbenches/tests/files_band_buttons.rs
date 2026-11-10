//! The answer buttons of the files band: the button itself stays inside its frame and the window.

#[path = "files_common/mod.rs"]
mod files_common;

use files_common::harness::Harness;
use files_common::{Shelf, widget};

const WIDTHS: [f32; 8] = [1300.0, 900.0, 720.0, 560.0, 400.0, 300.0, 200.0, 160.0];

/// The button itself (not its frame) stays inside its frame and the window, at every width, for the
/// answers of the confirm question, of a taken name and of carrying.
fn button_is_inside(h: &Harness, what: &str, width: f32) {
    let (fl, _, fr, _) = h.edges(what);
    let (bl, _, br, _) = h.button_edges(what);
    assert!(
        bl >= fl - 0.5 && br <= fr + 0.5,
        "{width} {what}: button {bl}..{br} is outside its frame {fl}..{fr}"
    );
    assert!(
        bl >= -0.5 && br <= width + 0.5,
        "{width} {what}: button {bl}..{br} is outside the window"
    );
}

#[test]
fn the_answer_buttons_are_inside_their_frame_and_the_window_at_every_width() {
    for width in WIDTHS {
        // Delete asks: "Keep it" and the answer.
        let (_, w) = widget(Shelf::new().file("a").file("b"));
        let mut h = Harness::new(w, width, 700.0);
        h.click("a");
        h.click("delete");
        for what in ["keep-it", "really"] {
            button_is_inside(&h, what, width);
        }
        drop(h); // one harness at a time
        // A taken name asks: "Back" and "Keep both".
        let (_, w) = widget(Shelf::new().file("a").file("b"));
        let mut h = Harness::new(w, width, 700.0);
        h.click("a");
        h.click("rename");
        h.press("line");
        h.type_text("b.txt\u{0}");
        h.widget.with_state(|s| s.set_input("b.txt"));
        h.rebuild();
        h.click("apply");
        for what in ["back", "keep-both"] {
            button_is_inside(&h, what, width);
        }
        drop(h);
        // Carrying: "Cancel" and the put-down button.
        let (_, w) = widget(Shelf::new().file("a").file("b"));
        let mut h = Harness::new(w, width, 700.0);
        h.click("a");
        h.widget
            .with_state(|s| s.carry(fs_components_workbenches::file_manager::CarryKind::Move));
        h.rebuild();
        button_is_inside(&h, "stop-carrying", width);
        if h.has("put-down") {
            button_is_inside(&h, "put-down", width);
        }
    }
}
