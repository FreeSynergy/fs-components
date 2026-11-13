//! The language entry of the standard menus: the menu lists the choices, a choice writes the program's setting and changes the
//! menus at once, a failed write is a visible problem, and the settings page reaches the same setting through the same object.
//! Fake environment as in `standard_menus.rs`; temporary folders only.

#[path = "menu_common/mod.rs"]
mod menu_common;

use std::collections::HashMap;
use std::sync::Arc;

use fs_components_app::{FileLanguageControl, LanguageChoice, LanguageControl, Scope, Status};
use menu_common::fixture::entry;
use menu_common::standard::{PROGRAM, SUB, fixture, labels, loc, menus};

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
