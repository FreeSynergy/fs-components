//! What the input tests of the menus have in common: a program with two menus and a surface that
//! counts presses.
//!
//! Tests: Test fixture.

#![allow(dead_code)]

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use blinc_layout::div::div;
use fs_components_app::{AppWidget, Element, Menu, MenuEntry, Screen, Surface};

use super::Harness;

pub const FILE: &str = "fs-app-menu-0";
pub const VIEW: &str = "fs-app-menu-1";
pub const PANEL: &str = "fs-app-menu-panel";

pub fn entry(id: &str) -> String {
    format!("fs-app-menu-entry-{id}")
}

/// A surface that fills its room and counts the presses that reach it.
#[derive(Default)]
pub struct Target(pub Arc<AtomicUsize>);

impl Surface for Target {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        let presses = Arc::clone(&self.0);
        Box::new(div().id("target").w(width).h(height).on_click(move |_| {
            presses.fetch_add(1, Ordering::SeqCst);
        }))
    }
}

pub fn menus() -> Vec<Menu> {
    vec![
        Menu::new(
            "File",
            vec![
                MenuEntry::new("first", "First"),
                MenuEntry::new("off", "Disabled").disabled(),
                MenuEntry::new("third", "Third"),
                MenuEntry::new("fourth", "Fourth"),
            ],
        ),
        Menu::new("View", vec![MenuEntry::new("show", "Show")]),
    ]
}

pub type Log = Arc<Mutex<Vec<String>>>;

pub fn app_with(title: &str, screens: &[&str], presses: &Arc<AtomicUsize>) -> (AppWidget, Log) {
    let screens = screens
        .iter()
        .map(|id| Screen::new(*id, *id, Arc::new(Target(Arc::clone(presses)))))
        .collect();
    let app = AppWidget::new(title, screens).unwrap();
    let log: Log = Arc::default();
    let chosen = Arc::clone(&log);
    app.set_menus(menus(), move |id| {
        chosen.lock().unwrap().push(id.to_owned());
    })
    .unwrap();
    (app, log)
}

pub fn harness(screens: &[&str]) -> (Harness, Log, Arc<AtomicUsize>) {
    let presses = Arc::default();
    let (app, log) = app_with("Title", screens, &presses);
    (Harness::new(app, 800.0, 500.0), log, presses)
}

pub fn open_menu(h: &Harness) -> Option<usize> {
    h.app.with_shell(fs_components_app::Shell::open_menu)
}
