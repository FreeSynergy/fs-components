//! `list-view` with very many rows, drawn offscreen with `fs_components_render::draw_to_png`.
//!
//! Pattern: Pixel tests — skipped (with a message) where there is no graphics adapter. They live
//! here because this crate has the widget and the renderer as dependencies. No window is opened.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use fs_components_render::{Canvas, RenderError, draw_to_png};
use fs_components_widgets::list_view::STYLES;
use fs_components_widgets::{Column, ColumnWidth, ListPosition, ListView, Row, RowSource};

/// Rows made up on request; counts what was asked of it.
struct Numbers {
    count: usize,
    rows_read: AtomicUsize,
    furthest: AtomicUsize,
}

impl Numbers {
    fn new(count: usize) -> Arc<Self> {
        Arc::new(Self {
            count,
            rows_read: AtomicUsize::new(0),
            furthest: AtomicUsize::new(0),
        })
    }
}

impl RowSource for Numbers {
    fn count(&self) -> usize {
        self.count
    }
    fn rows(&self, first: usize, limit: usize) -> Result<Vec<Row>, String> {
        let end = (first + limit).min(self.count);
        self.rows_read.fetch_add(end - first, Ordering::Relaxed);
        self.furthest.fetch_max(end, Ordering::Relaxed);
        Ok((first..end)
            .map(|i| {
                Row::new(
                    format!("k{i}"),
                    vec![format!("Row {i}"), format!("{}", i * 2)],
                )
            })
            .collect())
    }
}

fn columns() -> Vec<Column> {
    vec![
        Column::new("Name", ColumnWidth::Share(2.0)),
        Column::new("Twice", ColumnWidth::Pixels(120.0)),
    ]
}

fn out(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("fs-list-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

/// Draw; `None` if this machine has no graphics adapter. `FS_COMP_SHOTS` names a folder to keep
/// the pictures in.
fn draw(list: ListView, name: &str) -> Option<image::RgbaImage> {
    let path = out(name);
    let canvas = Canvas::new(500, 300, STYLES);
    match draw_to_png(&canvas, |w, h| list.build(w, h), || {}, &path) {
        Err(RenderError::NoGraphics(why)) => {
            eprintln!("skipped: {why}");
            None
        }
        other => {
            other.unwrap();
            if let Some(dir) = std::env::var_os("FS_COMP_SHOTS") {
                std::fs::create_dir_all(&dir).unwrap();
                std::fs::copy(&path, PathBuf::from(dir).join(name)).unwrap();
            }
            Some(image::open(&path).unwrap().to_rgba8())
        }
    }
}

/// Which pixel rows hold something else than the row background, in the Name column.
fn ink_rows(picture: &image::RgbaImage) -> usize {
    let corner = picture
        .get_pixel(picture.width() - 2, picture.height() - 2)
        .0;
    (0..picture.height())
        .filter(|y| (5..300).any(|x| picture.get_pixel(x, *y).0 != corner))
        .count()
}

#[test]
fn one_hundred_thousand_rows_read_only_what_is_in_view() {
    let source = Numbers::new(100_000);
    let list = ListView::new(columns(), source.clone());
    let Some(picture) = draw(list, "top.png") else {
        return;
    };
    assert!(ink_rows(&picture) > 60, "rows are drawn");
    assert!(
        source.rows_read.load(Ordering::Relaxed) <= 128,
        "read {} rows",
        source.rows_read.load(Ordering::Relaxed)
    );
}

#[test]
fn the_middle_of_a_long_list_shows_rows_and_reads_only_there() {
    let source = Numbers::new(100_000);
    let position = ListPosition::new();
    position.scroll_to_row(50_000);
    let list = ListView::new(columns(), source.clone()).position(&position);
    let Some(picture) = draw(list, "middle.png") else {
        return;
    };
    assert!(ink_rows(&picture) > 60, "the rows at 50,000 are drawn");
    assert!(source.rows_read.load(Ordering::Relaxed) <= 128);
    assert!(source.furthest.load(Ordering::Relaxed) >= 50_000);
    assert!((position.offset() - 50_000.0 * 32.0).abs() < 1.0);
}

#[test]
fn the_end_of_a_long_list_is_reached_and_not_passed() {
    let source = Numbers::new(100_000);
    let position = ListPosition::new();
    position.scroll_to_row(usize::MAX);
    let list = ListView::new(columns(), source.clone()).position(&position);
    let Some(picture) = draw(list, "end.png") else {
        return;
    };
    assert!(ink_rows(&picture) > 60);
    assert_eq!(source.furthest.load(Ordering::Relaxed), 100_000);
}

#[test]
fn ten_million_rows_do_not_crash() {
    let source = Numbers::new(10_000_000);
    let position = ListPosition::new();
    position.scroll_to_row(9_000_000);
    let list = ListView::new(columns(), source).position(&position);
    let Some(picture) = draw(list, "ten-million.png") else {
        return;
    };
    assert!(ink_rows(&picture) > 50);
}

#[test]
fn no_rows_shows_the_callers_text_and_a_row_that_fails_shows_why() {
    struct Failing;
    impl RowSource for Failing {
        fn count(&self) -> usize {
            3
        }
        fn rows(&self, _first: usize, _limit: usize) -> Result<Vec<Row>, String> {
            Err("the disk is gone".into())
        }
    }
    let empty = Numbers::new(0);
    let list = ListView::new(columns(), empty).when_empty("Nothing here");
    if draw(list, "empty.png").is_none() {
        return;
    }
    let failing = ListView::new(columns(), Arc::new(Failing));
    let picture = draw(failing, "failed.png").unwrap();
    assert!(ink_rows(&picture) > 0);
}
