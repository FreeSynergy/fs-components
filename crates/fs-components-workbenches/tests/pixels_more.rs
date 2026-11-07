//! More pixel tests: marked table cells and the search field.
//!
//! Tests: Pixel tests — skipped (with a message) where there is no graphics adapter or no fonts.

mod common;

use std::sync::Arc;

use common::draw::{Long, Scratch, fonts_work, pixels};
use fs_components_workbenches::{Block, Inline, Mark, MarkKind, Picture, snapshot};

#[test]
fn marked_cells_in_a_narrow_table_wrap_instead_of_overprinting() {
    let tmp = Scratch::new();
    if !fonts_work(&tmp) {
        eprintln!("skipped: no fonts or no graphics");
        return;
    }
    // Eight columns, each cell a link of 24 characters (the length up to which a run used to
    // refuse to wrap): they cannot all fit at 1000 px.
    let cell = |c: char| {
        let text = c.to_string().repeat(24);
        let end = text.len();
        Inline::new(
            text,
            vec![Mark::new(MarkKind::Link { href: "x".into() }, 0, end)],
        )
    };
    let row = ['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h']
        .iter()
        .map(|c| cell(*c))
        .collect();
    let blocks = vec![Block::Table {
        align: vec![],
        rows: vec![row],
    }];
    let file = tmp.file("links.png");
    if snapshot(Picture::new(Arc::new(Long(blocks)), 1000, 400), &file).is_err() {
        eprintln!("skipped: no graphics");
        return;
    }
    let img = pixels(&file);
    // Links are blue, not dark: look for any pixel that is not background-ish (light grey/white).
    let inked = |p: &image::Rgba<u8>| p[0] < 200 || p[1] < 200 || p[2] < 200;
    let spans = {
        let mut spans: Vec<(u32, u32)> = Vec::new();
        for x in 340..img.width() - 8 {
            if (110..300).any(|y| inked(img.get_pixel(x, y)) && !is_border(&img, x, y)) {
                match spans.last_mut() {
                    Some(s) if x - s.1 <= 6 => s.1 = x,
                    _ => spans.push((x, x)),
                }
            }
        }
        spans
    };
    assert!(
        spans.len() >= 8,
        "cells overprint each other: only {} groups: {spans:?}",
        spans.len()
    );
    assert!(
        spans.last().unwrap().1 <= 966,
        "the last cell runs past the table edge: {spans:?}"
    );
}

/// The table's border lines and header background are light grey; text is much darker or blue.
fn is_border(img: &image::RgbaImage, x: u32, y: u32) -> bool {
    let p = img.get_pixel(x, y);
    let grey = p[0].abs_diff(p[1]) < 8 && p[1].abs_diff(p[2]) < 8;
    grey && p[0] > 150
}

#[test]
fn the_search_field_is_a_small_one() {
    let tmp = Scratch::new();
    let file = tmp.file("field.png");
    if snapshot(Picture::new(Arc::new(common::Fake), 900, 300), &file).is_err() {
        eprintln!("skipped: no graphics");
        return;
    }
    let img = pixels(&file);
    // Column 200 is inside the field, right of the placeholder: only its borders mark it.
    let bg = *img.get_pixel(200, 2);
    let rows: Vec<u32> = (0..140).filter(|&y| *img.get_pixel(200, y) != bg).collect();
    let height = rows.last().unwrap() - rows.first().unwrap();
    // The medium field measured 59 px (border and shadow included); the small one 53.
    assert!(
        height < 56,
        "the field is {height} px tall: not the small size"
    );
    assert!(height > 20, "the field cannot be found: {rows:?}");
}
