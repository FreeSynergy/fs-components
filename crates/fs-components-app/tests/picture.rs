//! The shell as a picture, drawn offscreen with the real view. No window is opened.
//!
//! Tests: Pixel tests — skipped (with a message) where there is no graphics adapter. Each surface
//! fills its area with one theme colour, so the test sees which surface is drawn and how big it is.

use std::path::PathBuf;
use std::sync::Arc;

use blinc_layout::div::div;
use fs_components_app::{AppError, AppWidget, Element, Screen, Status, Surface};
use fs_components_render::{Canvas, RenderError, draw_to_png};
use image::{Rgba, RgbaImage};

const SIZE: (u32, u32) = (600, 400);

struct Block(&'static str);

impl Surface for Block {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        Box::new(div().class(self.0).w(width).h(height))
    }
    fn styles(&self) -> String {
        ".t-red { background: var(--error-bg); } .t-yellow { background: var(--warning-bg); }"
            .to_owned()
    }
}

fn app(names: &[&'static str]) -> AppWidget {
    let screens = names
        .iter()
        .map(|n| Screen::new(*n, *n, Arc::new(Block(n))))
        .collect();
    AppWidget::new("Title", screens).unwrap()
}

/// Draw and read back; `None` if this machine has no graphics adapter.
fn draw(app: &AppWidget, name: &str) -> Option<RgbaImage> {
    let dir = std::env::temp_dir().join(format!("fs-app-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path: PathBuf = dir.join(name);
    match app.picture(SIZE.0, SIZE.1, &path) {
        Err(AppError::Picture(RenderError::NoGraphics(why))) => {
            eprintln!("skipped: {why}");
            None
        }
        other => {
            other.unwrap();
            Some(image::open(&path).unwrap().to_rgba8())
        }
    }
}

fn at(picture: &RgbaImage, x: u32, y: u32) -> Rgba<u8> {
    *picture.get_pixel(x, y)
}

#[test]
fn selecting_a_screen_draws_its_surface() {
    let app = app(&["t-red", "t-yellow"]);
    let Some(first) = draw(&app, "first.png") else {
        return;
    };
    assert!(app.select("t-yellow"));
    let second = draw(&app, "second.png").unwrap();
    assert_ne!(at(&first, 400, 200), at(&second, 400, 200));
}

#[test]
fn the_surface_is_told_the_room_beside_the_navigation() {
    let app = app(&["t-red", "t-yellow"]);
    let Some(picture) = draw(&app, "room.png") else {
        return;
    };
    let surface = at(&picture, 400, 200);
    assert_eq!(
        at(&picture, 202, 50),
        surface,
        "starts after the navigation"
    );
    assert_eq!(
        at(&picture, 598, 365),
        surface,
        "reaches the right and the status line"
    );
    assert_ne!(
        at(&picture, 100, 200),
        surface,
        "the navigation is not covered"
    );
    assert_ne!(at(&picture, 400, 20), surface, "the bar is not covered");
    assert_ne!(
        at(&picture, 400, 390),
        surface,
        "the status line is not covered"
    );
}

#[test]
fn one_screen_needs_no_navigation() {
    let app = app(&["t-red"]);
    let Some(picture) = draw(&app, "single.png") else {
        return;
    };
    assert_eq!(at(&picture, 2, 200), at(&picture, 400, 200));
}

#[test]
fn a_problem_changes_the_status_line_but_not_the_surface() {
    let app = app(&["t-yellow"]);
    let Some(before) = draw(&app, "before.png") else {
        return;
    };
    app.set_status(Some(Status::Problem("failed".into())));
    let after = draw(&app, "after.png").unwrap();
    assert_ne!(at(&before, 590, 390), at(&after, 590, 390));
    assert_eq!(at(&before, 400, 200), at(&after, 400, 200));
}

/// The window draws through `build`, whose areas are live; a picture draws the areas once. Both
/// must show the same, also after a change.
#[test]
fn the_live_areas_draw_what_the_picture_draws() {
    let app = app(&["t-red", "t-yellow"]);
    assert!(app.select("t-yellow"));
    app.set_status(Some(Status::Problem("failed".into())));
    let Some(picture) = draw(&app, "once.png") else {
        return;
    };
    let path = std::env::temp_dir().join(format!("fs-app-live-{}.png", std::process::id()));
    let canvas = Canvas::new(SIZE.0, SIZE.1, app.styles());
    draw_to_png(&canvas, |w, h| app.build(w, h), || {}, &path).unwrap();
    let live = image::open(&path).unwrap().to_rgba8();
    for (x, y) in [(400, 200), (100, 200), (400, 20), (590, 390)] {
        assert_eq!(at(&live, x, y), at(&picture, x, y), "at {x},{y}");
    }
}

/// A surface that uses its shell while it is built, and remembers the size it was told.
struct Nosy {
    app: std::sync::OnceLock<AppWidget>,
    told: std::sync::Mutex<(f32, f32)>,
}

impl Surface for Nosy {
    fn build(&self, width: f32, height: f32) -> Box<dyn Element> {
        *self.told.lock().unwrap() = (width, height);
        if let Some(app) = self.app.get() {
            app.set_status(Some(Status::Info("seen".into())));
            app.with_shell(|shell| assert_eq!(shell.title(), "Title"));
        }
        Box::new(div())
    }
    fn styles(&self) -> String {
        if let Some(app) = self.app.get() {
            app.with_shell(|shell| assert_eq!(shell.title(), "Title"));
        }
        String::new()
    }
}

fn nosy() -> (AppWidget, Arc<Nosy>) {
    let surface = Arc::new(Nosy {
        app: std::sync::OnceLock::new(),
        told: std::sync::Mutex::new((-1.0, -1.0)),
    });
    let app = AppWidget::new(
        "Title",
        vec![Screen::new(
            "a",
            "a",
            Arc::clone(&surface) as Arc<dyn Surface>,
        )],
    )
    .unwrap();
    assert!(surface.app.set(app.clone()).is_ok());
    (app, surface)
}

#[test]
fn a_surface_may_use_its_shell_while_it_is_built() {
    let (app, _surface) = nosy();
    let _ = app.styles();
    if draw(&app, "nosy.png").is_some() {
        app.with_shell(|shell| {
            assert_eq!(shell.status(), Some(&Status::Info("seen".into())));
        });
    }
}

#[test]
fn a_surface_is_never_told_a_negative_size() {
    let (app, surface) = nosy();
    let path = std::env::temp_dir().join(format!("fs-app-tiny-{}.png", std::process::id()));
    match app.picture(100, 50, &path) {
        Err(AppError::Picture(RenderError::NoGraphics(why))) => eprintln!("skipped: {why}"),
        other => {
            other.unwrap();
            assert_eq!(*surface.told.lock().unwrap(), (100.0, 0.0));
        }
    }
}

#[test]
fn a_long_status_stays_in_its_line() {
    let app = app(&["t-yellow"]);
    let Some(before) = draw(&app, "short.png") else {
        return;
    };
    app.set_status(Some(Status::Problem("failed because ".repeat(200))));
    let after = draw(&app, "long.png").unwrap();
    assert_eq!(at(&before, 400, 360), at(&after, 400, 360));
}
