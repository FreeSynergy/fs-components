//! SKELETON of the language gate (docs/language-and-frontend-menu.md, section 2): the same window drawn once per language.
//!
//! Tests: Pixel tests — skipped (with a message) where there is no graphics adapter. Pictures of both languages stay in
//! `FS_COMP_SHOTS` if that is set (the "images of both" for the strategy: `shell-de.png`, `shell-en.png`).
//!
//! Skeleton, named: the language comes from a table in this file, because the `Locale` API of the design does not exist yet.
//! When it does, `bar` takes a `Locale` and the loop below runs over `["de", "en"]` with `LANG` unset and the setting
//! given explicitly (a test never reads the real environment). The comparison against stored pictures follows with
//! `fs-components-pictures` (branch frontend-umbau-3).

#[path = "menu_common/mod.rs"]
mod menu_common;

use std::path::PathBuf;
use std::sync::Arc;

use blinc_layout::div::div;
use fs_components_app::{AppWidget, Element, Menu, MenuEntry, Screen, Surface};
use fs_components_render::{Canvas, RenderError, draw_to_png};
use image::RgbaImage;
use menu_common::composed;

const SIZE: (u32, u32) = (640, 240);

struct Blank;

impl Surface for Blank {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        Box::new(div().class("t-blank").w(width).h(height))
    }
    fn styles(&self) -> String {
        ".t-blank { background: var(--error-bg); }".to_owned()
    }
}

/// The menu bar of the standard shell (ADR 0065) in one language: Program, View, Help.
fn bar(lang: &str) -> Vec<Menu> {
    let t = |en: &'static str, de: &'static str| if lang == "de" { de } else { en };
    vec![
        Menu::new(t("Program", "Programm"), vec![MenuEntry::new("quit", t("Quit", "Beenden"))]),
        Menu::new(t("View", "Ansicht"), vec![MenuEntry::new("view", t("Tiles", "Kacheln"))]),
        Menu::new(t("Help", "Hilfe"), vec![MenuEntry::new("about", t("About", "Über"))]),
    ]
}

fn draw(lang: &str) -> Option<RgbaImage> {
    let dir = std::env::var_os("FS_COMP_SHOTS").map_or_else(
        || std::env::temp_dir().join(format!("fs-app-lang-{}", std::process::id())),
        PathBuf::from,
    );
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("shell-{lang}.png"));
    let app = AppWidget::new("Title", vec![Screen::new("one", "First", Arc::new(Blank))]).unwrap();
    app.set_menus(bar(lang), |_| {}).unwrap();
    let canvas = Canvas::new(SIZE.0, SIZE.1, app.styles());
    match draw_to_png(&canvas, |w, h| composed(&app, w, h), || {}, &path) {
        Err(RenderError::NoGraphics(why)) => {
            eprintln!("skipped: {why}");
            None
        }
        other => {
            other.unwrap();
            let picture = image::open(&path).unwrap().to_rgba8();
            if std::env::var_os("FS_COMP_SHOTS").is_none() {
                std::fs::remove_file(&path).unwrap();
            }
            Some(picture)
        }
    }
}

#[test]
fn the_same_window_in_german_and_english_differs_in_the_bar_and_nowhere_else() {
    let _one = menu_common::serial();
    let (Some(de), Some(en)) = (draw("de"), draw("en")) else {
        return;
    };
    let bar_differs = (0..SIZE.0).any(|x| de.get_pixel(x, 22) != en.get_pixel(x, 22));
    assert!(bar_differs, "the words of the bar are drawn in the language");
    for (x, y) in [(100, 200), (400, 200), (630, 230)] {
        assert_eq!(de.get_pixel(x, y), en.get_pixel(x, y), "the work area at {x},{y} is the same");
    }
}
