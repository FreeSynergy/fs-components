//! Editors wrap their text at the real width of their box, and a block is as tall as its text (finding of 2026-11-14: the
//! Confluence editor broke lines at about 250 px in a box of about 500 px; the Notion block cut long text off).
//!
//! Tests: pixel tests on the pictures of the window, skipped where there is no graphics adapter or no font. Pictures are
//! kept when `FS_COMP_SHOTS` names a directory. The words are short, so a line that wraps more than one word before the
//! edge of the box is a line that wrapped at a width that is not the box's.

mod common;

use std::sync::Arc;

use common::Editable;
use common::draw::{Scratch, dark, fonts_work, pixels};
use fs_components_workbenches::{Layout, Picture, Step, snapshot};

/// `words` short words in one line, the way a person types a paragraph.
fn prose(words: usize) -> String {
    (0..words)
        .map(|i| ["lorem", "ipsum", "dolor", "sit", "amet", "elit"][i % 6])
        .collect::<Vec<_>>()
        .join(" ")
}

fn shoot(layout: Layout, width: u32, text: &str, name: &str) -> Option<image::RgbaImage> {
    let tmp = Scratch::new();
    if !fonts_work(&tmp) {
        eprintln!("skipped: no fonts or no graphics");
        return None;
    }
    let source = Editable::with(&[("handbook.md", text)]);
    source
        .heading_titles
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let mut picture = Picture::new(Arc::new(source), width, 700);
    picture.layout = layout;
    picture.steps = vec![Step::Open("handbook.md".into()), Step::Edit];
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

/// The box of the editor: its border is a light grey line. (left, right, top, bottom) of the first box whose top edge
/// is at or below `below`, found on the row/column through `probe`.
fn border(p: image::Rgba<u8>) -> bool {
    p[0] < 245 && p[0] > 150 && p[0].abs_diff(p[1]) < 8
}

/// The slack, in pixels, between the right edge of the box and the ink of the longest text line inside it, for a box
/// through the point (`x`, `y`) that lies inside it; `x` is a column right of every line of text (the scan up and down
/// along it meets only the frame, whatever the font).
fn slack(img: &image::RgbaImage, x: u32, y: u32) -> (u32, u32, u32) {
    let w = img.width();
    let right = (x..w)
        .find(|&i| border(*img.get_pixel(i, y)))
        .expect("right edge");
    let left = (0..x)
        .rev()
        .find(|&i| border(*img.get_pixel(i, y)))
        .expect("left edge");
    let top = (0..y)
        .rev()
        .find(|&j| border(*img.get_pixel(x, j)))
        .expect("top edge");
    let bottom = (y..img.height())
        .find(|&j| border(*img.get_pixel(x, j)))
        .expect("bottom edge");
    let mut ink = left;
    for j in top + 3..bottom - 2 {
        for i in left + 3..right - 2 {
            if dark(*img.get_pixel(i, j)) {
                ink = ink.max(i);
            }
        }
    }
    (right - ink, left, right)
}

#[test]
fn the_confluence_editor_wraps_at_the_right_edge_of_its_box() {
    let Some(img) = shoot(
        Layout::Confluence,
        1400,
        &format!("# T\n\n{}\n", prose(60)),
        "confluence-umbruch.png",
    ) else {
        return;
    };
    // Inside the box, below the first lines.
    let (slack, left, right) = slack(&img, 995, 400);
    assert!(right - left > 400, "the box is wide: {left}..{right}");
    assert!(
        slack < 90,
        "lines wrap at the edge of the box (slack {slack} px of {})",
        right - left
    );
}

#[test]
fn a_notion_block_wraps_at_the_right_edge_and_shows_all_its_lines() {
    let Some(img) = shoot(
        Layout::Notion,
        1400,
        &format!("# T\n\n{}\n", prose(150)),
        "notion-umbruch.png",
    ) else {
        return;
    };
    let (slack, left, right) = slack(&img, 1130, 420);
    assert!(
        slack < 90,
        "lines wrap at the edge of the block (slack {slack} px of {})",
        right - left
    );
}
