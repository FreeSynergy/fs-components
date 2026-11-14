//! A narrow window keeps the page readable (finding E5 of 2026-11-14: at 700 px the middle column of the Confluence layout was
//! 130 px wide and the title "a.md" broke letter by letter), and the buttons of the editor are right (finding E4: Cancel was as
//! blue as Save).
//!
//! Tests: pixel tests on the pictures of the layouts (skipped without graphics or fonts). A title is the big text under the top
//! line: one line of ink at any width, never one line per letter.

mod common;

use std::sync::Arc;

use common::Editable;
use common::draw::{Scratch, fonts_work, pixels};
use fs_components_workbenches::{Layout, Picture, Step, snapshot};

fn shoot(layout: Layout, width: u32, steps: Vec<Step>, name: &str) -> Option<image::RgbaImage> {
    let tmp = Scratch::new();
    if !fonts_work(&tmp) {
        eprintln!("skipped: no fonts or no graphics");
        return None;
    }
    let source = Editable::with(&[("a.md", "# A\n\nText\n")]);
    let mut picture = Picture::new(Arc::new(source), width, 400);
    picture.layout = layout;
    picture.steps = steps;
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

/// The tallest run of dark rows right of `from_x` (the title is the biggest text): its height and the width of its ink.
fn title_ink(img: &image::RgbaImage, from_x: u32, from_y: u32) -> (u32, u32) {
    let dark = |x: u32, y: u32| {
        let p = img.get_pixel(x, y);
        p[0] < 110 && p[1] < 110 && p[2] < 110
    };
    let mut best = (0, 0, 0);
    let mut start = None;
    for y in from_y..=img.height() {
        let ink = y < img.height() && (from_x..img.width()).any(|x| dark(x, y));
        match (ink, start) {
            (true, None) => start = Some(y),
            (false, Some(s)) => {
                if y - s > best.0 {
                    best = (y - s, s, y);
                }
                start = None;
            }
            _ => {}
        }
    }
    let cols: Vec<u32> = (from_x..img.width())
        .filter(|&x| (best.1..best.2).any(|y| dark(x, y)))
        .collect();
    let width = cols.last().zip(cols.first()).map_or(0, |(l, f)| l - f);
    (best.0, width)
}

#[test]
fn the_title_never_breaks_letter_by_letter_in_a_narrow_window() {
    for layout in [Layout::Confluence, Layout::Notion] {
        for width in [900, 700, 600, 480] {
            let name = format!("schmal-{layout:?}-{width}.png").to_lowercase();
            let Some(img) = shoot(layout, width, vec![Step::Open("a.md".into())], &name) else {
                return;
            };
            // The title is the biggest text: one line of "a.md" is about 85 px wide and 30 px high; a title that breaks per
            // letter is a column of single letters, each line about 20 px wide.
            let left = if width >= 700 { 300 } else { 150 };
            let (height, ink) =
                title_ink(&img, left, if layout == Layout::Notion { 190 } else { 60 });
            assert!(
                ink >= 60,
                "{layout:?} at {width}: the title is one line (ink {ink} px wide, {height} px high)"
            );
        }
    }
}

#[test]
fn cancel_is_not_as_blue_as_save() {
    let Some(img) = shoot(
        Layout::Confluence,
        1000,
        vec![Step::Open("a.md".into()), Step::Edit],
        "editor-knoepfe.png",
    ) else {
        return;
    };
    let blue = |x0: u32, x1: u32| {
        (100..170)
            .flat_map(|y| (x0..x1).map(move |x| (x, y)))
            .filter(|&(x, y)| {
                let p = img.get_pixel(x, y);
                p[2] > 200 && p[0] < 100 && p[1] < 150
            })
            .count()
    };
    // Save at about 358..421, Cancel at about 439..515 on the bar of the editor.
    assert!(blue(350, 430) > 500, "Save is the primary button");
    assert_eq!(blue(432, 530), 0, "Cancel is an outline button");
}
