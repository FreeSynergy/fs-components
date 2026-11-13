//! The standard menus of the shell (ADR 0065): structure in both languages, the check marks, the next level (a submenu), the
//! callbacks, and the language choice written to a temporary folder. Input tests through Blinc's router; no window.
//!
//! The environment is a fake one (`LANG=de_DE.UTF-8` or `LANG=en_US.UTF-8`): a test never reads the real one and never writes
//! outside its temporary folder.

#[path = "menu_common/mod.rs"]
mod menu_common;

use std::sync::Arc;

use fs_components_app::{AppWidget, FrontendItem};
use menu_common::fixture::{PANEL, entry};
use menu_common::standard::{Calls, SUB, VIEW, fixture, labels, menus};

#[test]
fn the_bar_speaks_one_language_german_or_english() {
    for (lang, bar, program, help) in [
        (
            "de_DE.UTF-8",
            ["Programm", "Ansicht", "Hilfe"],
            ["Einstellungen", "Sprache", "Beenden"],
            ["Hilfe", "Über"],
        ),
        (
            "en_US.UTF-8",
            ["Program", "View", "Help"],
            ["Settings", "Language", "Quit"],
            ["Help", "About"],
        ),
    ] {
        let f = fixture("bar", lang);
        let m = menus(&f.h.app);
        assert_eq!(
            m.iter().map(|m| m.label.as_str()).collect::<Vec<_>>(),
            bar,
            "{lang}"
        );
        assert_eq!(labels(&m[0].entries), program, "{lang}");
        assert_eq!(labels(&m[2].entries), help, "{lang}");
        let _ = std::fs::remove_dir_all(f.dir);
    }
}

#[test]
fn views_stay_apart_from_frontends_and_the_frontends_are_a_next_level_with_the_active_one_checked()
{
    let f = fixture("frontends", "en_US.UTF-8");
    let view = &menus(&f.h.app)[1];
    assert_eq!(
        labels(&view.entries),
        ["Tiles", "List", "Frontends"],
        "the program's views first, then the entry Frontends"
    );
    assert!(
        view.entries[..2]
            .iter()
            .all(|e| e.children.is_empty() && !e.checked),
        "views are plain lines"
    );
    let sub = &view.entries[2].children;
    assert_eq!(labels(sub), ["Classic", "Board", "Day list"]);
    assert_eq!(
        sub.iter().map(|e| e.checked).collect::<Vec<_>>(),
        [true, false, false]
    );
    assert!(
        sub[2].disabled,
        "a frontend that cannot be chosen is shown locked"
    );
    assert!(
        !f.h.app.choose("fs.frontends"),
        "the entry Frontends opens a level, it is no choice"
    );
    assert!(f.calls.lock().unwrap().is_empty());
    let _ = std::fs::remove_dir_all(f.dir);
}

#[test]
fn a_press_opens_the_next_level_beside_the_panel_and_a_second_press_closes_it() {
    let mut f = fixture("levels", "en_US.UTF-8");
    f.h.click(VIEW);
    assert!(f.h.has(PANEL) && !f.h.has(SUB));
    f.h.click(&entry("fs.frontends"));
    assert!(f.h.has(SUB), "the next level is open");
    let (px, _, pw, _) = f.h.bounds(PANEL);
    let (sx, sy, _, _) = f.h.bounds(SUB);
    assert!(
        sx >= px + pw - 1.0,
        "to the right of the panel: {sx} against {}",
        px + pw
    );
    let (_, ly, _, lh) = f.h.bounds(&entry("fs.frontends"));
    assert!(
        sy >= ly - lh && sy <= ly + lh,
        "level with its line: {sy} against {ly}"
    );
    assert!(f.h.has(&entry("fs.frontend.board")), "its lines are there");
    assert_eq!(
        f.h.app.with_shell(fs_components_app::Shell::open_sub),
        Some(2)
    );
    f.h.click(&entry("fs.frontends"));
    assert_eq!(f.h.app.with_shell(fs_components_app::Shell::open_sub), None);
    f.h.settle();
    assert!(!f.h.has(SUB), "closed again");
    assert!(f.h.has(PANEL), "the menu itself stays open");
    let _ = std::fs::remove_dir_all(f.dir);
}

#[test]
fn choosing_a_frontend_calls_back_moves_the_check_and_closes_the_menu() {
    let mut f = fixture("choose", "en_US.UTF-8");
    f.h.click(VIEW);
    f.h.click(&entry("fs.frontends"));
    let point = f.h.centre(&entry("fs.frontend.board"));
    f.h.press_at(point);
    f.h.frame();
    assert_eq!(*f.calls.lock().unwrap(), ["frontend:board"]);
    let sub = &menus(&f.h.app)[1].entries[2].children;
    assert_eq!(
        sub.iter().map(|e| e.checked).collect::<Vec<_>>(),
        [false, true, false]
    );
    assert_eq!(
        f.h.app.with_shell(fs_components_app::Shell::open_menu),
        None
    );
    // A locked frontend is not chosen.
    assert!(!f.h.app.choose("fs.frontend.day"));
    assert_eq!(f.calls.lock().unwrap().len(), 1);
    // A view of the program goes to the program's callback, not to the frontend one.
    assert!(f.h.app.choose("tiles"));
    assert_eq!(*f.calls.lock().unwrap(), ["frontend:board", "entry:tiles"]);
    let _ = std::fs::remove_dir_all(f.dir);
}

#[test]
fn the_program_can_change_its_frontends_while_the_window_runs() {
    let f = fixture("hot", "de_DE.UTF-8");
    let seen: Calls = Arc::default();
    let s = Arc::clone(&seen);
    f.h.app
        .set_frontends(
            vec![
                FrontendItem::new("a", "Alpha"),
                FrontendItem::new("b", "Beta"),
            ],
            "b",
            move |id| s.lock().unwrap().push(id.to_owned()),
        )
        .unwrap();
    let sub = &menus(&f.h.app)[1].entries[2].children;
    assert_eq!((labels(sub), sub[1].checked), (vec!["Alpha", "Beta"], true));
    assert_eq!(menus(&f.h.app)[1].entries[2].label, "Frontends");
    assert!(f.h.app.choose("fs.frontend.a"));
    assert_eq!(*seen.lock().unwrap(), ["a"], "the new callback answers");
    assert!(f.calls.lock().unwrap().is_empty(), "the old one does not");
    f.h.app.set_active_frontend("b").unwrap();
    assert!(menus(&f.h.app)[1].entries[2].children[1].checked);
    // Without standard menus there is nothing to change.
    let bare = AppWidget::new("T", vec![]).unwrap();
    assert!(bare.set_frontends(vec![], "x", |_| {}).is_err());
    let _ = std::fs::remove_dir_all(f.dir);
}

#[test]
fn quit_closes_the_window_and_the_words_of_the_shell_are_paired() {
    let f = fixture("quit", "en_US.UTF-8");
    let closed = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let c = Arc::clone(&closed);
    f.h.app.set_quit(move || {
        c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    });
    assert!(f.h.app.choose("fs.quit"));
    assert_eq!(closed.load(std::sync::atomic::Ordering::SeqCst), 1);
    assert!(
        f.calls.lock().unwrap().is_empty(),
        "the program is not asked about Quit"
    );
    assert_eq!(
        fs_components_mf2::check_pairs(fs_components_app::TEXTS),
        Vec::<String>::new()
    );
    let _ = std::fs::remove_dir_all(f.dir);
}
