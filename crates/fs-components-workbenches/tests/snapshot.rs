//! The picture: drawn offscreen, with the real view.
//!
//! Pattern: Pixel tests — skipped (with a message) where there is no graphics adapter. They look
//! at what is on the picture, not only that something is.

mod common;

use std::path::PathBuf;
use std::sync::Arc;

use common::Fake;
use fs_components_workbenches::{
    Block, Document, DocumentSource, Entry, Hit, Inline, LinkTarget, Mark, MarkKind, Picture,
    SourceError, Step, snapshot,
};

fn out(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("fs-wb-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

fn pixels(path: &PathBuf) -> image::RgbaImage {
    image::open(path).unwrap().to_rgba8()
}

/// A source with one long document.
struct Long(Vec<Block>);

impl DocumentSource for Long {
    fn entries(&self, _p: &str) -> Result<Vec<Entry>, SourceError> {
        Ok(vec![Entry::new("long", "Long")])
    }
    fn read(&self, path: &str) -> Result<Document, SourceError> {
        Ok(Document::new(path, self.0.clone()))
    }
    fn resolve(&self, _f: &str, h: &str) -> LinkTarget {
        LinkTarget::Missing(h.into())
    }
    fn find(&self, _t: &str) -> Result<Vec<Hit>, SourceError> {
        Ok(vec![])
    }
}

fn paragraphs(n: usize) -> Vec<Block> {
    (0..n)
        .map(|i| {
            Block::Paragraph(Inline::plain(format!(
                "Paragraph number {i} of the long document."
            )))
        })
        .collect()
}

#[test]
fn snapshot_draws_something_and_refuses_size_zero() {
    let mut picture = Picture::new(Arc::new(Fake), 400, 300);
    picture.steps = vec![Step::Open("a".into()), Step::Search("dir".into())];
    let file = out("s.png");
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
    let source = Arc::new(Long(paragraphs(200)));
    let top_file = out("top.png");
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
    let down_file = out("down.png");
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
    let source = Arc::new(Long(paragraphs(3000)));
    let file = out("many.png");
    match snapshot(Picture::new(source, 700, 400), &file) {
        Err(e) => eprintln!("skipped: {e}"),
        Ok(s) => assert!(s.scroll.content_height > s.scroll.viewport_height),
    }
}

#[test]
fn a_long_word_in_marked_text_stays_inside_the_window() {
    let word = "x".repeat(120);
    let text = format!("start {word} end");
    let blocks = vec![Block::Paragraph(Inline::new(
        text,
        vec![Mark::new(MarkKind::Strong, 0, 5)],
    ))];
    let file = out("word.png");
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
    let make = |kind: Option<MarkKind>| {
        let marks = kind.map(|k| vec![Mark::new(k, 0, 5)]).unwrap_or_default();
        vec![Block::Paragraph(Inline::new("Words in the text", marks))]
    };
    let draw = |name: &str, kind: Option<MarkKind>| {
        let file = out(name);
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
