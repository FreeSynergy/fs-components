//! The page icons of the Notion page tree stand out from the row they sit on.
//!
//! Tests: pixel test, skipped where there is no graphics adapter or no font. The measure is the contrast of WCAG 2.x
//! (`MIN_ICON_CONTRAST`, the least for a part of a picture): enough pixels of the icon must reach it against the
//! background of the row. The picture is kept when `FS_COMP_SHOTS` names a directory.

mod common;

use std::sync::Arc;

use common::Editable;
use common::draw::{Scratch, fonts_work, pixels};
use fs_components_styles::Rgba;
use fs_components_widgets::icon::MIN_ICON_CONTRAST;
use fs_components_workbenches::{Layout, Picture, Step, snapshot};

fn rgba(p: image::Rgba<u8>) -> Rgba {
    Rgba {
        r: p[0],
        g: p[1],
        b: p[2],
        a: 255,
    }
}

/// Pixels of the icon box (20 columns from `x0`, rows y-9..y+9) that reach the contrast against the row's own
/// background (the pixel at x = 250, right of every title), for the rows whose middle line is `ys`.
fn strong_pixels(img: &image::RgbaImage, x0: u32, y: u32) -> usize {
    let bg = rgba(*img.get_pixel(250, y));
    (y - 9..=y + 9)
        .flat_map(|yy| (x0..x0 + 20).map(move |x| (x, yy)))
        .filter(|&(x, yy)| rgba(*img.get_pixel(x, yy)).contrast(bg) >= MIN_ICON_CONTRAST)
        .count()
}

#[test]
fn the_child_icon_in_the_page_tree_is_not_pale() {
    let tmp = Scratch::new();
    if !fonts_work(&tmp) {
        eprintln!("skipped: no fonts or no graphics");
        return;
    }
    let source = Editable::with(&[
        ("handbook.md", "# Handbook\n\nThe handbook.\n"),
        ("handbook/chapter-one.md", "# Chapter one\n\nText.\n"),
        ("notes.md", "# Notes\n\nText.\n"),
    ]);
    source
        .heading_titles
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let mut picture = Picture::new(Arc::new(source), 1000, 650);
    picture.layout = Layout::Notion;
    picture.steps = vec![
        Step::Open("handbook.md".into()),
        Step::Expand("handbook.md".into()),
    ];
    let file = tmp.file("seitenbaum-notion-icons.png");
    if snapshot(picture, &file).is_err() {
        eprintln!("skipped: no graphics");
        return;
    }
    if let Some(dir) = std::env::var_os("FS_COMP_SHOTS") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(
            &file,
            std::path::Path::new(&dir).join("seitenbaum-notion-icons.png"),
        )
        .unwrap();
    }
    let img = pixels(&file);
    // Rows of the tree: Handbook (selected) at y 114, Chapter one at 142, Notes at 170.
    let counts = [(30, 114), (48, 142), (30, 170)].map(|(x, y)| strong_pixels(&img, x, y));
    eprintln!("strong pixels per row: {counts:?}");
    // The outline of a 16 px sheet alone is about 40 pixels; a pale icon has a handful.
    for (row, n) in ["Handbook", "Chapter one", "Notes"].iter().zip(counts) {
        assert!(
            n >= 25,
            "the icon of {row} is pale: {n} pixels reach the contrast"
        );
    }
}
