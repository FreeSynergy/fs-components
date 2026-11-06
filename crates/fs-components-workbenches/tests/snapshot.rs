//! The picture: drawn offscreen, with the real view.
//!
//! Pattern: Pixel tests — skipped (with a message) where there is no graphics adapter. They look
//! at what is on the picture, not only that something is.

mod common;

use std::sync::Arc;

use common::Fake;
use common::draw::{Long, Scratch, fonts_work, paragraphs, pixels};
use fs_components_workbenches::{Block, Inline, Mark, MarkKind, Picture, Step, snapshot};

#[test]
fn snapshot_draws_something_and_refuses_size_zero() {
    let tmp = Scratch::new();
    let mut picture = Picture::new(Arc::new(Fake), 400, 300);
    picture.steps = vec![Step::Open("a".into()), Step::Search("dir".into())];
    let file = tmp.file("s.png");
    match snapshot(picture, &file) {
        Err(e) => eprintln!("skipped: {e}"),
        Ok(s) => {
            assert_eq!(s.reader.current_path(), Some("a"));
            let img = pixels(&file);
            assert_eq!(img.dimensions(), (400, 300));
            let first = *img.get_pixel(0, 0);
            assert!(img.pixels().any(|p| *p != first), "the picture is blank");
        }
    }
    assert!(snapshot(Picture::new(Arc::new(Fake), 0, 300), &file).is_err());
}

#[test]
fn a_long_document_has_a_bounded_viewport_and_scrolls() {
    let tmp = Scratch::new();
    if !fonts_work(&tmp) {
        eprintln!("skipped: no fonts or no graphics");
        return;
    }
    let source = Arc::new(Long(paragraphs(200)));
    let top_file = tmp.file("top.png");
    let Ok(t) = snapshot(Picture::new(source.clone(), 700, 400), &top_file) else {
        eprintln!("skipped: no graphics");
        return;
    };
    assert!(
        t.scroll.viewport_height > 0.0 && t.scroll.viewport_height < 400.0,
        "{:?}",
        t.scroll
    );
    assert!(
        t.scroll.content_height > 5.0 * t.scroll.viewport_height,
        "the content must be taller than its window: {:?}",
        t.scroll
    );
    let mut down = Picture::new(source, 700, 400);
    down.steps = vec![Step::Scroll(1500)];
    let down_file = tmp.file("down.png");
    let d = snapshot(down, &down_file).unwrap();
    assert!(
        d.scroll.offset > 1000.0,
        "it must have scrolled: {:?}",
        d.scroll
    );
    let (a, b) = (pixels(&top_file), pixels(&down_file));
    let differing = a.pixels().zip(b.pixels()).filter(|(x, y)| x != y).count();
    assert!(
        differing > 1000,
        "scrolling must change the picture ({differing} pixels)"
    );
}
#[test]
fn thousands_of_paragraphs_draw_without_a_crash() {
    let tmp = Scratch::new();
    let source = Arc::new(Long(paragraphs(3000)));
    let file = tmp.file("many.png");
    match snapshot(Picture::new(source, 700, 400), &file) {
        Err(e) => eprintln!("skipped: {e}"),
        Ok(s) => assert!(s.scroll.content_height > s.scroll.viewport_height),
    }
}

#[test]
fn a_long_word_in_marked_text_stays_inside_the_window() {
    let tmp = Scratch::new();
    if !fonts_work(&tmp) {
        eprintln!("skipped: no fonts or no graphics");
        return;
    }
    let word = "x".repeat(120);
    let text = format!("start {word} end");
    let blocks = vec![Block::Paragraph(Inline::new(
        text,
        vec![Mark::new(MarkKind::Strong, 0, 5)],
    ))];
    let file = tmp.file("word.png");
    let picture = Picture::new(Arc::new(Long(blocks)), 700, 300);
    let Ok(_) = snapshot(picture, &file) else {
        eprintln!("skipped: no graphics");
        return;
    };
    let img = pixels(&file);
    // The page area starts right of the 320 px sidebar; the last 12 columns must stay empty.
    let edge = img.width() - 12;
    let bg = *img.get_pixel(img.width() - 1, 150);
    for y in 80..img.height() {
        for x in edge..img.width() {
            assert_eq!(*img.get_pixel(x, y), bg, "ink at the right edge ({x},{y})");
        }
    }
}

#[test]
fn emphasis_and_strong_look_different_from_plain_text() {
    let tmp = Scratch::new();
    if !fonts_work(&tmp) {
        eprintln!("skipped: no fonts or no graphics");
        return;
    }
    let make = |kind: Option<MarkKind>| {
        // "Plain" still goes through the flow path (an empty mark), so the comparison is fair.
        let marks = kind
            .map(|k| vec![Mark::new(k, 0, 5)])
            .unwrap_or_else(|| vec![Mark::new(MarkKind::Strong, 3, 3)]);
        vec![Block::Paragraph(Inline::new("Words in the text", marks))]
    };
    let draw = |name: &str, kind: Option<MarkKind>| {
        let file = tmp.file(name);
        snapshot(Picture::new(Arc::new(Long(make(kind))), 700, 200), &file).ok()?;
        Some(pixels(&file))
    };
    let (Some(plain), Some(em), Some(strong)) = (
        draw("plain.png", None),
        draw("em.png", Some(MarkKind::Emphasis)),
        draw("strong.png", Some(MarkKind::Strong)),
    ) else {
        eprintln!("skipped: no graphics");
        return;
    };
    let diff = |a: &image::RgbaImage, b: &image::RgbaImage| {
        a.pixels().zip(b.pixels()).filter(|(x, y)| x != y).count()
    };
    assert!(diff(&plain, &em) > 20, "emphasis is drawn like plain text");
    assert!(
        diff(&plain, &strong) > 20,
        "strong is drawn like plain text"
    );
    assert!(diff(&em, &strong) > 20, "emphasis and strong look the same");
}
