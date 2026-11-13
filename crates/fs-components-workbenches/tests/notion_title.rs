//! Notion layout, editing a page whose text starts with a level-1 heading: the title is drawn once.
//!
//! Tests: pixel test, skipped where there is no graphics adapter or no font. The block editor holds the heading as its
//! first block; the big title above is then left out. Rows with big dark glyphs (the 40 px title) must not appear
//! above the first block, and the heading text appears only once.

mod common;

use std::sync::Arc;

use common::Editable;
use common::draw::{Scratch, fonts_work, pixels};
use fs_components_workbenches::{Layout, Picture, Step, snapshot};

#[test]
fn the_title_is_not_drawn_twice_while_editing() {
    let tmp = Scratch::new();
    if !fonts_work(&tmp) {
        eprintln!("skipped: no fonts or no graphics");
        return;
    }
    let source = Arc::new(Editable::with(&[("a", "# a\n\nBody text\n")]));
    let mut picture = Picture::new(source, 1000, 520);
    picture.layout = Layout::Notion;
    picture.steps = vec![Step::Open("a".into()), Step::Edit];
    let file = tmp.file("notion-edit.png");
    if snapshot(picture, &file).is_err() {
        eprintln!("skipped: no graphics");
        return;
    }
    if let Some(dir) = std::env::var_os("FS_COMP_SHOTS") {
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::copy(&file, std::path::Path::new(&dir).join("notion-edit.png")).unwrap();
    }
    // The big title would be a band of dark glyphs about 30 px high; count the rows that hold dark ink in the column.
    let img = pixels(&file);
    let dark_rows = (0..img.height())
        .filter(|&y| (300..760).any(|x| common::draw::dark(*img.get_pixel(x, y))))
        .count();
    let tall = (0..img.height())
        .filter(|&y| (300..760).any(|x| common::draw::dark(*img.get_pixel(x, y))))
        .collect::<Vec<_>>();
    // Longest run of consecutive dark rows: a 40 px title is a run of 20 or more; 14 px text stays below that.
    let mut longest = 0;
    let mut run = 0;
    let mut last = None;
    for y in tall {
        run = if last == Some(y - 1) { run + 1 } else { 1 };
        last = Some(y);
        longest = longest.max(run);
    }
    assert!(dark_rows > 0, "nothing drawn");
    assert!(
        longest < 20,
        "a big title is drawn above the blocks ({longest} rows)"
    );
}
