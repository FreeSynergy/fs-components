//! The menus in the state of the shell, without a window and without Blinc's runtime.
//!
//! Tests: State tests — transitions only. A hang here is a deadlock, not slowness.

use std::sync::{Arc, Mutex};

use blinc_layout::div::div;
use fs_components_app::{
    AppError, AppWidget, Element, Menu, MenuEntry, Screen, Shell, Status, Surface,
};

struct Empty;

impl Surface for Empty {
    fn build(&self, _w: f32, _h: f32) -> Box<dyn Element> {
        Box::new(div())
    }
}

fn screens() -> Vec<Screen> {
    vec![
        Screen::new("a", "A", Arc::new(Empty)),
        Screen::new("b", "B", Arc::new(Empty)),
    ]
}

fn menus() -> Vec<Menu> {
    vec![
        Menu::new(
            "File",
            vec![
                MenuEntry::new("open", "Open"),
                MenuEntry::new("save", "Save").disabled(),
            ],
        ),
        Menu::new("Nothing", Vec::new()),
        Menu::new("Go", vec![MenuEntry::new("go-b", "To B")]),
    ]
}

#[test]
fn a_shell_starts_without_menus_and_no_menu_can_open() {
    let mut shell = Shell::new("T", screens()).unwrap();
    assert_eq!(shell.menus(), []);
    assert!(!shell.open(0));
    assert_eq!(shell.open_menu(), None);
    assert!(shell.handler_for("open").is_none());
}

#[test]
fn two_entries_with_one_id_are_refused_and_nothing_changes() {
    let mut shell = Shell::new("T", screens()).unwrap();
    shell.set_menus(menus(), |_| {}).unwrap();
    let twice = vec![
        Menu::new("X", vec![MenuEntry::new("same", "One")]),
        Menu::new("Y", vec![MenuEntry::new("same", "Two")]),
    ];
    assert_eq!(
        shell.set_menus(twice, |_| {}),
        Err(AppError::DuplicateMenuEntry("same".to_owned()))
    );
    assert_eq!(shell.menus(), menus());
}

#[test]
fn one_menu_is_open_at_a_time_and_an_empty_one_never() {
    let mut shell = Shell::new("T", screens()).unwrap();
    shell.set_menus(menus(), |_| {}).unwrap();
    assert!(shell.open(0));
    assert!(!shell.open(1), "no entries");
    assert_eq!(shell.open_menu(), Some(0), "the refusal changed nothing");
    assert!(!shell.open(9));
    assert!(shell.open(2));
    assert_eq!(shell.open_menu(), Some(2));
    shell.close_menu();
    assert_eq!(shell.open_menu(), None);
}

#[test]
fn new_menus_close_the_open_one() {
    let mut shell = Shell::new("T", screens()).unwrap();
    shell.set_menus(menus(), |_| {}).unwrap();
    assert!(shell.open(0));
    shell.set_menus(menus(), |_| {}).unwrap();
    assert_eq!(shell.open_menu(), None);
}

#[test]
fn only_a_known_enabled_entry_has_a_handler() {
    let mut shell = Shell::new("T", screens()).unwrap();
    shell.set_menus(menus(), |_| {}).unwrap();
    assert!(shell.handler_for("open").is_some());
    assert!(shell.handler_for("save").is_none(), "disabled");
    assert!(shell.handler_for("nope").is_none(), "unknown");
}

#[test]
fn the_handler_receives_the_id_and_may_use_the_shell() {
    let app = AppWidget::new("T", screens()).unwrap();
    let seen = Arc::new(Mutex::new(Vec::new()));
    let (this, log) = (app.clone(), Arc::clone(&seen));
    app.set_menus(menus(), move |id| {
        log.lock().unwrap().push(id.to_owned());
        assert!(this.select("b"));
        this.set_status(Some(Status::Info(id.to_owned())));
        this.with_shell(|shell| assert_eq!(shell.title(), "T"));
    })
    .unwrap();
    assert!(app.choose("go-b"));
    assert!(!app.choose("save"), "disabled");
    assert!(!app.choose("nope"), "unknown");
    assert_eq!(*seen.lock().unwrap(), ["go-b"]);
    app.with_shell(|shell| {
        assert_eq!(shell.current().unwrap().id, "b");
        assert_eq!(shell.status(), Some(&Status::Info("go-b".into())));
    });
}

#[test]
fn the_handler_may_replace_the_menus_while_it_runs() {
    let app = AppWidget::new("T", screens()).unwrap();
    let this = app.clone();
    app.set_menus(menus(), move |_| {
        let gone = vec![Menu::new(
            "File",
            vec![MenuEntry::new("open", "Open").disabled()],
        )];
        this.set_menus(gone, |_| {}).unwrap();
    })
    .unwrap();
    assert!(app.choose("open"));
    assert!(!app.choose("open"), "disabled by the first time");
}
