//! Drawing offscreen.
//!
//! Pattern: Pixel tests — skipped (with a message) where there is no graphics adapter. No window
//! is opened.

use std::path::PathBuf;

use blinc_layout::div::div;
use fs_components_render::{Canvas, RenderError, draw_to_png};

fn out(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("fs-render-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

#[test]
fn an_empty_canvas_is_refused() {
    let canvas = Canvas::new(0, 10, "");
    let result = draw_to_png(&canvas, |_, _| div(), || {}, &out("empty.png"));
    assert_eq!(result, Err(RenderError::EmptyCanvas));
}

#[test]
fn the_style_sheet_reaches_the_picture_and_after_layout_runs_once() {
    let path = out("box.png");
    let css = ".t-box { background: var(--error-bg); }";
    let canvas = Canvas::new(40, 30, css);
    let mut ran = 0;
    let result = draw_to_png(
        &canvas,
        |w, h| {
            assert!((w - 40.0).abs() < f32::EPSILON && (h - 30.0).abs() < f32::EPSILON);
            div().w(w).h(h).class("t-box")
        },
        || ran += 1,
        &path,
    );
    if let Err(RenderError::NoGraphics(why)) = &result {
        eprintln!("skipped: {why}");
        return;
    }
    result.unwrap();
    assert_eq!(ran, 1);
    let picture = image::open(&path).unwrap().to_rgba8();
    assert_eq!(picture.dimensions(), (40, 30));
    let centre = picture.get_pixel(20, 15).0;
    assert_ne!(centre[3], 0, "the box is drawn");
    assert!(
        picture.pixels().all(|p| p.0 == centre),
        "one colour everywhere"
    );
}
