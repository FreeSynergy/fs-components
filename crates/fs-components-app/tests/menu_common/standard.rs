//! What the tests of the standard menus have in common: a window with the standard menus in the language a fake `LANG` gives,
//! over temporary setting files.
//!
//! Tests: Test fixture.

#![allow(dead_code)]

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use fs_components_app::{
    AppWidget, FileLanguageControl, FrontendItem, LanguageControl, Menu, Screen, StandardCallbacks,
    StandardMenus,
};
use fs_components_words::{LanguageSetting, Locale};

use super::Harness;
use super::fixture::Target;

pub const SUB: &str = "fs-app-menu-sub-panel";
pub const PROGRAM: &str = "fs-app-menu-0";
pub const VIEW: &str = "fs-app-menu-1";

pub fn loc(tag: &str) -> Locale {
    Locale::parse(tag).unwrap()
}

pub fn temp(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("fs-app-standard-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

pub fn control(dir: &std::path::Path, lang: &str) -> Arc<FileLanguageControl> {
    let env: HashMap<String, String> = HashMap::from([("LANG".to_owned(), lang.to_owned())]);
    Arc::new(FileLanguageControl::new(
        Some(LanguageSetting::new(dir.join("own").join("language"))),
        Some(LanguageSetting::new(dir.join("shared"))),
        Arc::new(env),
        vec![loc("de"), loc("en")],
    ))
}

pub type Calls = Arc<Mutex<Vec<String>>>;

pub struct Fixture {
    pub h: Harness,
    pub calls: Calls,
    pub dir: PathBuf,
    pub control: Arc<FileLanguageControl>,
}

pub fn frontends() -> Vec<FrontendItem> {
    vec![
        FrontendItem::new("classic", "Classic"),
        FrontendItem::new("board", "Board"),
        FrontendItem::new("day", "Day list").disabled(),
    ]
}

/// A window with the standard menus in the language the fake `LANG` gives.
pub fn fixture(name: &str, lang: &str) -> Fixture {
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

pub fn menus(app: &AppWidget) -> Vec<Menu> {
    app.with_shell(|s| s.menus().to_vec())
}

pub fn labels(entries: &[fs_components_app::MenuEntry]) -> Vec<&str> {
    entries.iter().map(|e| e.label.as_str()).collect()
}
