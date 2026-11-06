//! Helpers for the pixel tests: scratch folder, a source with one document, pixel lookups.
//!
//! Pattern: Test helpers.

#![allow(dead_code)]

use std::path::PathBuf;

use fs_components_workbenches::{
    Block, Document, DocumentSource, Entry, Hit, Inline, LinkTarget, SourceError,
};

/// A folder for the pictures of one test, removed when the test ends (also when it fails).
pub struct Scratch(PathBuf);

impl Scratch {
    pub fn new() -> Self {
        static N: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let n = N.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("fs-wb-{}-{n}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    pub fn file(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

pub fn pixels(path: &PathBuf) -> image::RgbaImage {
    image::open(path).unwrap().to_rgba8()
}

/// A source with one long document.
pub struct Long(pub Vec<Block>);

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

pub fn paragraphs(n: usize) -> Vec<Block> {
    (0..n)
        .map(|i| {
            Block::Paragraph(Inline::plain(format!(
                "Paragraph number {i} of the long document."
            )))
        })
        .collect()
}

/// Dark text pixel (backgrounds and borders of the table are light).
pub fn dark(p: &image::Rgba<u8>) -> bool {
    p[0] < 120 && p[1] < 120 && p[2] < 120
}

/// First row (y) with dark text inside the x range, from `from_y` on.
pub fn ink_top(img: &image::RgbaImage, xs: std::ops::Range<u32>, from_y: u32) -> Option<u32> {
    (from_y..img.height()).find(|&y| xs.clone().any(|x| dark(img.get_pixel(x, y))))
}

/// X ranges that contain dark text between `y0..y1`, separated by more than 14 empty columns.
pub fn ink_spans(
    img: &image::RgbaImage,
    xs: std::ops::Range<u32>,
    y0: u32,
    y1: u32,
) -> Vec<(u32, u32)> {
    ink_spans_gap(img, xs, y0, y1, 14)
}

/// Like [`ink_spans`] with the number of empty columns that separates two spans.
pub fn ink_spans_gap(
    img: &image::RgbaImage,
    xs: std::ops::Range<u32>,
    y0: u32,
    y1: u32,
    gap: u32,
) -> Vec<(u32, u32)> {
    let has = |x: u32| (y0..y1).any(|y| dark(img.get_pixel(x, y)));
    let mut spans: Vec<(u32, u32)> = Vec::new();
    for x in xs {
        if has(x) {
            match spans.last_mut() {
                Some(s) if x - s.1 <= gap => s.1 = x,
                _ => spans.push((x, x)),
            }
        }
    }
    spans
}

/// Do glyphs draw at all on this machine? (With no fonts installed nothing does; tests that look
/// at text then have nothing to look at and skip.)
pub fn fonts_work(tmp: &Scratch) -> bool {
    let file = tmp.file("fonts-probe.png");
    let source = std::sync::Arc::new(Long(vec![Block::Paragraph(Inline::plain("Hello world"))]));
    if fs_components_workbenches::snapshot(
        fs_components_workbenches::Picture::new(source, 700, 200),
        &file,
    )
    .is_err()
    {
        return false;
    }
    let img = pixels(&file);
    (80..200).any(|y| (340..690).any(|x| dark(img.get_pixel(x, y))))
}
