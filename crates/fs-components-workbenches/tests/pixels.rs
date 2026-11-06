//! Pixel tests of cells and glyphs: what is on the picture, not only that something is.
//!
//! Pattern: Pixel tests — skipped (with a message) where there is no graphics adapter.

mod common;

use std::sync::Arc;

use common::draw::{Long, Scratch, ink_spans, ink_top, pixels};
use fs_components_workbenches::{Block, Inline, Mark, MarkKind, Picture, snapshot};

#[test]
fn all_cells_of_a_row_start_at_the_same_height() {
    let tmp = Scratch::new();
    let long = "word ".repeat(40);
    let row = vec![
        Inline::new("sortable", vec![Mark::new(MarkKind::Code, 0, 8)]),
        Inline::plain("data"),
        Inline::plain(long),
        Inline::plain("end"),
    ];
    let blocks = vec![Block::Table {
        align: vec![],
        rows: vec![row],
    }];
    let file = tmp.file("cells.png");
    if snapshot(Picture::new(Arc::new(Long(blocks)), 1000, 400), &file).is_err() {
        eprintln!("skipped: no graphics");
        return;
    }
    let img = pixels(&file);
    // The table's first row lies at about y 110 to 300; look at the page area right of the sidebar.
    let spans = ink_spans(&img, 340..img.width() - 20, 110, 300);
    let tops: Vec<u32> = spans
        .iter()
        .filter_map(|(a, b)| ink_top(&img, *a..*b + 1, 110))
        .collect();
    assert!(
        tops.len() >= 3,
        "expected several cells with text: {spans:?}"
    );
    let (min, max) = (tops.iter().min().unwrap(), tops.iter().max().unwrap());
    assert!(max - min <= 6, "cells start at different heights: {tops:?}");
}

#[test]
fn symbols_and_scripts_without_a_default_glyph_are_drawn_not_boxes() {
    let tmp = Scratch::new();
    let draw = |name: &str, text: &str| {
        let file = tmp.file(name);
        snapshot(
            Picture::new(
                Arc::new(Long(vec![Block::Paragraph(Inline::plain(text))])),
                700,
                200,
            ),
            &file,
        )
        .ok()?;
        Some(pixels(&file))
    };
    // U+1F4D8 and U+1F30D are not in Blinc's bundled emoji subset; U+10FFFD has no glyph anywhere.
    let (Some(book), Some(boxes), Some(deva), Some(deva_boxes)) = (
        draw("book.png", "\u{1F4D8} \u{1F30D}"),
        draw("boxes.png", "\u{10FFFD} \u{10FFFC}"),
        draw(
            "deva.png",
            "\u{0928}\u{092E}\u{0938}\u{094D}\u{0924}\u{0947}",
        ),
        draw("deva2.png", "\u{10FFFD}\u{10FFFC}\u{10FFFB}\u{10FFFA}"),
    ) else {
        eprintln!("skipped: no graphics");
        return;
    };
    let diff = |a: &image::RgbaImage, b: &image::RgbaImage| {
        a.pixels().zip(b.pixels()).filter(|(x, y)| x != y).count()
    };
    assert!(
        diff(&book, &boxes) > 50,
        "the emoji look like the empty box"
    );
    assert!(
        diff(&deva, &deva_boxes) > 50,
        "the script looks like empty boxes"
    );
    // Two emoji must not be drawn on top of each other: their ink spans two glyph widths.
    let spans = ink_spans(&book, 340..book.width() - 20, 80, 160);
    let (first, last) = (spans.first().unwrap().0, spans.last().unwrap().1);
    assert!(last - first >= 28, "the glyphs pile up: {spans:?}");
}
