//! The page tree of the wiki window in both layouts: a sub-page sits below its parent, indented, and the arrow in
//! front of the parent does not move the parent's title.
//!
//! Tests: pixel tests, skipped where there is no graphics adapter or no font. The pictures are kept when the
//! environment variable `FS_COMP_SHOTS` names a directory.

mod common;

use std::sync::Arc;

use common::Editable;
use common::draw::{Scratch, fonts_work, pixels};
use fs_components_workbenches::{Layout, Picture, Step, snapshot};

/// Rows (y0, y1, first inked column) of the page list, top to bottom, from `from_y` on. Columns left of `SKIP` (the
/// arrow of a folder) are not looked at; the row's own background is the pixel at `SKIP`.
const SKIP: u32 = 26;

fn rows(img: &image::RgbaImage, from_y: u32) -> Vec<(u32, u32, u32)> {
    let mut out: Vec<(u32, u32, u32)> = Vec::new();
    for y in from_y..img.height() {
        let bg = img.get_pixel(SKIP, y).0;
        let first = (SKIP + 1..250).find(|&x| {
            let p = img.get_pixel(x, y).0;
            (0..3)
                .map(|i| (i32::from(p[i]) - i32::from(bg[i])).abs())
                .sum::<i32>()
                > 60
        });
        match (first, out.last_mut()) {
            (Some(x), Some(r)) if y - r.1 <= 3 => {
                r.1 = y;
                r.2 = r.2.min(x);
            }
            (Some(x), _) => out.push((y, y, x)),
            (None, _) => {}
        }
    }
    out
}

fn picture(layout: Layout, name: &str) -> Option<image::RgbaImage> {
    let tmp = Scratch::new();
    if !fonts_work(&tmp) {
        eprintln!("skipped: no fonts or no graphics");
        return None;
    }
    let source = Editable::with(&[
        ("handbook.md", "# Handbook\n\nThe handbook.\n"),
        ("handbook/chapter-one.md", "# Chapter one\n\nText.\n"),
        ("notes.md", "# Notes\n\nText.\n"),
    ]);
    source
        .heading_titles
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let source = Arc::new(source);
    let mut picture = Picture::new(source, 1000, 650);
    picture.layout = layout;
    picture.steps = vec![
        Step::Open("handbook.md".into()),
        Step::Expand("handbook.md".into()),
    ];
    let file = tmp.file(name);
    if snapshot(picture, &file).is_err() {
        eprintln!("skipped: no graphics");
        return None;
    }
    if let Some(dir) = std::env::var_os("FS_COMP_SHOTS") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(&file, std::path::Path::new(&dir).join(name)).unwrap();
    }
    Some(pixels(&file))
}

/// The three rows Handbook, Chapter one, Notes of the picture: the first inked column of each (the icon).
fn icons(img: &image::RgbaImage, from_y: u32) -> [u32; 3] {
    let found = rows(img, from_y);
    assert!(found.len() >= 3, "fewer than three rows: {found:?}");
    [found[0].2, found[1].2, found[2].2]
}

fn check(layout: Layout, name: &str, from_y: u32) {
    let Some(img) = picture(layout, name) else {
        return;
    };
    let [parent, child, sibling] = icons(&img, from_y);
    assert!(
        child >= parent + 10,
        "{name}: the sub-page is not indented (parent {parent}, child {child})"
    );
    assert!(
        parent.abs_diff(sibling) <= 1,
        "{name}: the arrow shifts the parent (parent {parent}, sibling {sibling})"
    );
}

#[test]
fn notion_sub_page_is_indented_below_its_parent() {
    check(Layout::Notion, "seitenbaum-notion.png", 100);
}

#[test]
fn confluence_sub_page_is_indented_below_its_parent() {
    check(Layout::Confluence, "seitenbaum-confluence.png", 108);
}
