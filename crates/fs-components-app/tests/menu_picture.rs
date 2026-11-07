//! The menus as pictures, drawn offscreen with the real view, opened through the router. No window is
//! opened.
//!
//! Tests: Pixel tests — skipped (with a message) where there is no graphics adapter. Pictures stay
//! in `FS_COMP_SHOTS` if that is set.

#[path = "menu_common/mod.rs"]
mod menu_common;

use std::path::PathBuf;
use std::sync::Arc;

use blinc_layout::div::div;
use fs_components_app::{AppWidget, Element, Menu, MenuEntry, Screen, Surface};
use fs_components_render::{Canvas, RenderError, draw_to_png};
use image::RgbaImage;
use menu_common::{Harness, composed};

const SIZE: (u32, u32) = (800, 500);

struct Red;

impl Surface for Red {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        Box::new(div().class("t-red").w(width).h(height))
    }
    fn styles(&self) -> String {
        ".t-red { background: var(--error-bg); }".to_owned()
    }
}

fn app(with_menus: bool) -> AppWidget {
    let screens = vec![
        Screen::new("one", "First", Arc::new(Red)),
        Screen::new("two", "Second", Arc::new(Red)),
    ];
    let app = AppWidget::new("Title", screens).unwrap();
    if with_menus {
        let file = vec![
            MenuEntry::new("open", "Open"),
            MenuEntry::new("save", "Save (nothing to save)").disabled(),
            MenuEntry::new("long", "A long label that takes more than one line to say"),
        ];
        let menus = vec![
            Menu::new("File", file),
            Menu::new("Help", vec![MenuEntry::new("about", "About")]),
        ];
        app.set_menus(menus, |_| {}).unwrap();
    }
    app
}

/// Draw as the window composes it, overlay layer included. `None` without a graphics adapter.
fn draw(app: &AppWidget, name: &str) -> Option<RgbaImage> {
    let dir = std::env::var_os("FS_COMP_SHOTS").map_or_else(
        || std::env::temp_dir().join(format!("fs-app-menu-{}", std::process::id())),
        PathBuf::from,
    );
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    let canvas = Canvas::new(SIZE.0, SIZE.1, app.styles());
    match draw_to_png(&canvas, |w, h| composed(app, w, h), || {}, &path) {
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
fn the_menus_change_the_bar_only_and_an_open_menu_lies_over_the_work_area() {
    #[allow(clippy::cast_precision_loss)]
    let (w, h) = (SIZE.0 as f32, SIZE.1 as f32);
    let without = {
        let _one = menu_common::serial();
        draw(&app(false), "menu-none.png")
    };
    let Some(without) = without else {
        return;
    };
    let mut harness = Harness::new(app(true), w, h);
    let closed = draw(&harness.app, "menu-closed.png").unwrap();
    for (x, y) in [(100, 200), (400, 200), (795, 495), (795, 20)] {
        assert_eq!(
            without.get_pixel(x, y),
            closed.get_pixel(x, y),
            "at {x},{y}"
        );
    }
    let changed = (0..SIZE.0).any(|x| without.get_pixel(x, 22) != closed.get_pixel(x, 22));
    assert!(changed, "the labels are in the bar");

    harness.click("fs-app-menu-0");
    let (px, py, pw, ph) = harness.bounds("fs-app-menu-panel");
    let open = draw(&harness.app, "menu-open.png").unwrap();
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let inside = ((px + pw - 12.0) as u32, (py + ph - 12.0) as u32);
    assert!(inside.0 > 200, "right of the navigation, over the surface");
    assert_ne!(
        closed.get_pixel(inside.0, inside.1),
        open.get_pixel(inside.0, inside.1),
        "the panel covers the surface"
    );
    assert_eq!(closed.get_pixel(700, 400), open.get_pixel(700, 400));
}
