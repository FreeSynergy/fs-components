//! Where the panels of the standard menus stand: the panel of a menu hangs under its entry in the bar, in a narrow window too, and
//! the panel of the next level stands beside it or folds to its left, never outside the window. (Blinc moves an overlay that would
//! cross the right edge to the left; the next level is an overlay of its own so that only it moves.) German and English.

#[path = "menu_common/mod.rs"]
mod menu_common;

use menu_common::fixture::{PANEL, entry};
use menu_common::standard::{SUB, fixture_in};

const LANGS: [&str; 2] = ["de_DE.UTF-8", "en_US.UTF-8"];

fn trigger(index: usize) -> String {
    format!("fs-app-menu-{index}")
}

#[test]
fn the_panel_hangs_under_its_entry_for_every_menu_in_a_window_of_640_px() {
    for lang in LANGS {
        let mut f = fixture_in("under", lang, 640.0);
        for index in 0..3 {
            f.h.click(&trigger(index));
            let (px, _, pw, _) = f.h.bounds(PANEL);
            let (tx, _, _, _) = f.h.bounds(&trigger(index));
            assert!((px - tx).abs() < 1.0, "{lang} menu {index}: panel at {px}, entry at {tx}");
            assert!(px + pw <= 640.0, "{lang} menu {index}: the panel is inside the window");
            f.h.click(&trigger(index));
            f.h.settle();
        }
        let _ = std::fs::remove_dir_all(f.dir);
    }
}

/// Open a menu and its next level; the bounds of the panel, its entry in the bar and the next level.
type Bounds = (f32, f32, f32, f32);

fn with_level(lang: &str, width: f32, menu: usize, line: &str) -> ((f32, f32), Bounds, Bounds) {
    let mut f = fixture_in("level", lang, width);
    f.h.click(&trigger(menu));
    f.h.click(&entry(line));
    let t = f.h.bounds(&trigger(menu));
    let found = ((t.0, t.1), f.h.bounds(PANEL), f.h.bounds(SUB));
    let _ = std::fs::remove_dir_all(f.dir);
    found
}

#[test]
fn the_next_level_does_not_move_the_panel_and_stays_inside_the_window() {
    for lang in LANGS {
        for (menu, line) in [(0, "fs.language"), (1, "fs.frontends")] {
            for width in [800.0, 640.0, 520.0] {
                let ((tx, _), panel, sub) = with_level(lang, width, menu, line);
                assert!((panel.0 - tx).abs() < 1.0, "{lang} {line} at {width}: the panel stays under its entry ({} against {tx})", panel.0);
                assert!(sub.0 >= 0.0 && sub.0 + sub.2 <= width, "{lang} {line} at {width}: next level inside the window: {sub:?}");
                let right = sub.0 >= panel.0 + panel.2 - 1.0;
                let left = sub.0 + sub.2 <= panel.0 + 1.0;
                let overlaps_only_when_no_room = width < 2.0 * 220.0 + panel.0.min(8.0) + 8.0 || panel.0 + 2.0 * 220.0 > width;
                assert!(right || left || overlaps_only_when_no_room, "{lang} {line} at {width}: beside the panel: {panel:?} {sub:?}");
                // Level with its line: the top of the next level is the top of the line (the first line is 8 px or so below the panel top).
                assert!(sub.1 >= panel.1 && sub.1 <= panel.1 + 3.0 * 45.0, "{lang} {line} at {width}: level with its line: {sub:?} {panel:?}");
            }
        }
    }
}

#[test]
fn where_the_right_has_no_room_the_next_level_folds_to_the_left_of_the_panel() {
    // English, View > Frontends in 640 px: the panel stands at about 281..501, the right side has no room (501 + 220 + 8 > 640)
    // and the left has (281 - 220 = 61).
    let (_, panel, sub) = with_level("en_US.UTF-8", 640.0, 1, "fs.frontends");
    assert!(sub.0 + sub.2 <= panel.0 + 1.0, "folded to the left: {panel:?} {sub:?}");
    assert!(sub.0 >= 8.0, "and inside the window: {sub:?}");
    // Where the right has room it stands on the right, in both languages.
    for lang in LANGS {
        let (_, panel, sub) = with_level(lang, 800.0, 1, "fs.frontends");
        assert!(sub.0 >= panel.0 + panel.2 - 1.0, "{lang}: on the right: {panel:?} {sub:?}");
    }
    // Where neither side has room (the German entry stands further left), it is kept inside the window, as Blinc keeps an overlay.
    let (_, _, sub) = with_level("de_DE.UTF-8", 520.0, 1, "fs.frontends");
    assert!(sub.0 >= 8.0 && sub.0 + sub.2 <= 520.0, "inside the window: {sub:?}");
}
