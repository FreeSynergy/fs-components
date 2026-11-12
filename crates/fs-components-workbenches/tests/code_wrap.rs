//! A long code line in the reader never wraps and never runs out of its block: in the plain reader
//! and in both wiki layouts it stays on one line inside a box that scrolls sideways.
//!
//! Tests: pixel tests of whole pictures, skipped where there is no graphics adapter or no font. The
//! picture with a 400-character line is compared with one of 900 characters (a box of the block's
//! width shows the same part of both) and with one whose code is empty (what differs is the code
//! line: one line high, and it stops before the window's edge). Not asserted: the scrollbar and
//! scrolling by hand.

mod common;

use std::sync::Arc;

use common::draw::{Long, Scratch, dark, fonts_work, pixels};
use fs_components_workbenches::{Block, Layout, Picture, snapshot};
use image::RgbaImage;

const W: u32 = 1100;
const H: u32 = 520;

fn picture(layout: Layout, code: &str, tmp: &Scratch, name: &str) -> Option<RgbaImage> {
    let blocks = vec![Block::Code {
        lang: String::new(),
        text: code.to_string(),
    }];
    let mut picture = Picture::new(Arc::new(Long(blocks)), W, H);
    picture.layout = layout;
    let file = tmp.file(name);
    snapshot(picture, &file).ok()?;
    Some(pixels(&file))
}

/// The rectangle (left, top, right, bottom) of the pixels that differ.
fn changed(first: &RgbaImage, second: &RgbaImage) -> Option<(u32, u32, u32, u32)> {
    let mut found: Option<(u32, u32, u32, u32)> = None;
    for (x, y, p) in first.enumerate_pixels() {
        if p != second.get_pixel(x, y) {
            found = Some(match found {
                None => (x, y, x, y),
                Some((left, top, right, low)) => {
                    (left.min(x), top.min(y), right.max(x), low.max(y))
                }
            });
        }
    }
    found
}

#[test]
fn a_long_code_line_stays_on_one_line_inside_its_block_in_every_layout() {
    let tmp = Scratch::new();
    if !fonts_work(&tmp) {
        eprintln!("skipped: no fonts or no graphics");
        return;
    }
    let long = "word ".repeat(80); // 400 characters
    let longer = "word ".repeat(180);
    for (layout, name) in [
        (Layout::Classic, "classic"),
        (Layout::Confluence, "confluence"),
        (Layout::Notion, "notion"),
    ] {
        let (Some(empty), Some(short), Some(longer_pic)) = (
            picture(layout, "", &tmp, &format!("{name}-empty.png")),
            picture(layout, &long, &tmp, &format!("{name}-a.png")),
            picture(layout, &longer, &tmp, &format!("{name}-b.png")),
        ) else {
            eprintln!("skipped: no graphics");
            return;
        };
        let (left, top, right, bottom) =
            changed(&empty, &short).unwrap_or_else(|| panic!("{name}: no code drawn"));
        assert!(
            bottom - top < 24,
            "{name}: the line wrapped (changed rows {top}..{bottom})"
        );
        assert!(
            right < W - 16,
            "{name}: the line runs to the window's edge (columns {left}..{right}), not cut at its block"
        );
        assert!(
            right - left > 150,
            "{name}: the code line is short (columns {left}..{right})"
        );
        // the scrollbar's thumb is as long as the line is; the dark text is what must not change
        let text = |img: &RgbaImage| -> Vec<bool> {
            (top..=bottom)
                .flat_map(|y| (0..W).map(move |x| (x, y)))
                .map(|(x, y)| dark(*img.get_pixel(x, y)))
                .collect()
        };
        assert!(
            text(&short) == text(&longer_pic),
            "{name}: a longer line changes the picture, so it is not held in a box of the block's width"
        );
    }
}
