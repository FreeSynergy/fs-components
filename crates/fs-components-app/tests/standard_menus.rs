//! The standard menus of the shell (ADR 0065): structure in both languages, the check marks, the next level (a submenu), the
//! callbacks, and the language choice written to a temporary folder. Input tests through Blinc's router; no window.
//!
//! The environment is a fake one (`LANG=de_DE.UTF-8` or `LANG=en_US.UTF-8`): a test never reads the real one and never writes
//! outside its temporary folder.

#[path = "menu_common/mod.rs"]
mod menu_common;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use fs_components_app::{
    AppWidget, FileLanguageControl, FrontendItem, LanguageChoice, LanguageControl, Menu, Scope,
    Screen, StandardCallbacks, StandardMenus, Status,
};
use fs_components_words::{LanguageSetting, Locale};
use menu_common::Harness;
use menu_common::fixture::{PANEL, Target, entry};

const SUB: &str = "fs-app-menu-sub-panel";
const PROGRAM: &str = "fs-app-menu-0";
const VIEW: &str = "fs-app-menu-1";

fn loc(tag: &str) -> Locale {
    Locale::parse(tag).unwrap()
}

fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("fs-app-standard-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn control(dir: &std::path::Path, lang: &str) -> Arc<FileLanguageControl> {
    let env: HashMap<String, String> = HashMap::from([("LANG".to_owned(), lang.to_owned())]);
    Arc::new(FileLanguageControl::new(
        Some(LanguageSetting::new(dir.join("own").join("language"))),
        Some(LanguageSetting::new(dir.join("shared"))),
        Arc::new(env),
        vec![loc("de"), loc("en")],
    ))
}

type Calls = Arc<Mutex<Vec<String>>>;

struct Fixture {
    h: Harness,
    calls: Calls,
    dir: PathBuf,
    control: Arc<FileLanguageControl>,
}

fn frontends() -> Vec<FrontendItem> {
    vec![
        FrontendItem::new("classic", "Classic"),
        FrontendItem::new("board", "Board"),
        FrontendItem::new("day", "Day list").disabled(),
    ]
}

/// A window with the standard menus in the language the fake `LANG` gives.
fn fixture(name: &str, lang: &str) -> Fixture {
    let dir = temp(name);
    let control = control(&dir, lang);
    let locale = control.resolve().chosen.locale;
    let presses = Arc::default();
    let app = AppWidget::new(
        "Title",
        vec![Screen::new(
            "one",
            "One",
            Arc::new(Target(Arc::clone(&presses))),
        )],
    )
    .unwrap();
    let calls: Calls = Arc::default();
    let (e, f, l) = (Arc::clone(&calls), Arc::clone(&calls), Arc::clone(&calls));
    let mut standard = StandardMenus::new(locale);
    standard.views = vec![
        fs_components_app::MenuEntry::new("tiles", "Tiles"),
        fs_components_app::MenuEntry::new("list", "List"),
    ];
    standard.frontends = frontends();
    standard.active_frontend = Some("classic".into());
    standard.language = Some(control.clone());
    let callbacks = StandardCallbacks::new(move |id| e.lock().unwrap().push(format!("entry:{id}")))
        .on_frontend(move |id| f.lock().unwrap().push(format!("frontend:{id}")))
        .on_language(move |lo| l.lock().unwrap().push(format!("language:{lo}")));
    app.set_standard_menus(standard, callbacks).unwrap();
    Fixture {
        h: Harness::new(app, 800.0, 500.0),
        calls,
        dir,
        control,
    }
}

fn menus(app: &AppWidget) -> Vec<Menu> {
    app.with_shell(|s| s.menus().to_vec())
}

fn labels(entries: &[fs_components_app::MenuEntry]) -> Vec<&str> {
    entries.iter().map(|e| e.label.as_str()).collect()
}

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
fn the_language_menu_lists_the_system_choice_and_each_language_with_the_one_in_use_checked() {
    let f = fixture("langmenu", "de_DE.UTF-8");
    let lang = &menus(&f.h.app)[0].entries[1];
    assert_eq!(
        labels(&lang.children),
        ["Systemsprache (Standard)", "Deutsch", "English"],
        "language names are in their own language"
    );
    assert_eq!(
        lang.children.iter().map(|e| e.checked).collect::<Vec<_>>(),
        [true, false, false],
        "nothing set: the system choice"
    );
    assert_eq!(f.control.resolve().chosen.locale.to_string(), "de-DE");
    let _ = std::fs::remove_dir_all(f.dir);
}

#[test]
fn choosing_a_language_writes_the_programs_setting_changes_the_menus_at_once_and_says_so() {
    let mut f = fixture("langset", "de_DE.UTF-8");
    f.h.click(PROGRAM);
    f.h.click(&entry("fs.language"));
    assert!(f.h.has(SUB));
    let point = f.h.centre(&entry("fs.language.en"));
    f.h.press_at(point);
    f.h.frame();
    // Written, in the program's own file (the shared one stays untouched).
    assert_eq!(
        std::fs::read_to_string(f.dir.join("own").join("language")).unwrap(),
        "en\n"
    );
    assert!(!f.dir.join("shared").exists());
    assert_eq!(*f.calls.lock().unwrap(), ["language:en"]);
    let m = menus(&f.h.app);
    assert_eq!(
        m.iter().map(|m| m.label.as_str()).collect::<Vec<_>>(),
        ["Program", "View", "Help"],
        "the bar changed at once"
    );
    let lang = &m[0].entries[1];
    assert_eq!(lang.label, "Language");
    assert_eq!(
        lang.children.iter().map(|e| e.checked).collect::<Vec<_>>(),
        [false, false, true],
        "English is checked"
    );
    let status = f.h.app.with_shell(|s| s.status().cloned());
    assert!(
        matches!(status, Some(Status::Info(t)) if t.contains("changes at the next start")),
        "said in the new language"
    );
    // The system choice removes the override again, and the environment (German) answers.
    f.h.app.choose("fs.language.system");
    assert!(!f.dir.join("own").join("language").exists());
    assert_eq!(menus(&f.h.app)[0].label, "Programm");
    assert_eq!(f.control.current(Scope::Program), LanguageChoice::System);
    let _ = std::fs::remove_dir_all(f.dir);
}

#[test]
fn a_language_that_cannot_be_saved_is_a_visible_problem_and_changes_nothing() {
    let f = fixture("langfail", "de_DE.UTF-8");
    // A file where the folder of the setting should be: the write fails.
    std::fs::write(f.dir.join("own"), "in the way").unwrap();
    assert!(f.h.app.choose("fs.language.en"));
    let status = f.h.app.with_shell(|s| s.status().cloned());
    assert!(
        matches!(status, Some(Status::Problem(ref t)) if t.starts_with("Die Sprache konnte nicht gespeichert werden")),
        "{status:?}"
    );
    assert_eq!(menus(&f.h.app)[0].label, "Programm", "still German");
    assert!(f.calls.lock().unwrap().is_empty(), "no language callback");
    let _ = std::fs::remove_dir_all(f.dir);
}

#[test]
fn the_settings_page_changes_all_programs_through_the_same_object() {
    let f = fixture("shared", "en_US.UTF-8");
    let spoken = f
        .control
        .choose(Scope::AllPrograms, &LanguageChoice::Language(loc("de")))
        .unwrap();
    assert_eq!(
        spoken.to_string(),
        "de",
        "the shared setting answers when the program has none"
    );
    assert_eq!(
        std::fs::read_to_string(f.dir.join("shared")).unwrap(),
        "de\n"
    );
    // The program's own choice beats it (the chain), and removing that gives the shared one back.
    assert_eq!(
        f.control
            .choose(Scope::Program, &LanguageChoice::Language(loc("en")))
            .unwrap()
            .to_string(),
        "en"
    );
    assert_eq!(
        f.control
            .choose(Scope::Program, &LanguageChoice::System)
            .unwrap()
            .to_string(),
        "de"
    );
    // No place for the setting: an error, nothing written.
    let none = FileLanguageControl::new(
        None,
        None,
        Arc::new(HashMap::<String, String>::new()),
        vec![loc("en")],
    );
    assert!(
        none.choose(Scope::Program, &LanguageChoice::System)
            .is_err()
    );
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
