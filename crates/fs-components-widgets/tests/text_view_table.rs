//! Tables show their column alignment (`:---:` centre, `:---` left, `---:` right) and show checkboxes: in task lists (GFM)
//! and in table cells (our extension; GFM has task items only in lists).
//!
//! Tests: the real element, drawn (skipped without fonts). Pictures are kept when `FS_COMP_SHOTS` names a directory.

mod common;

use common::{draw, has_fonts, serial};
use fs_components_widgets::TextView;
use fs_components_widgets::text::{Align, Block, Inline, ListItem};

const W: u32 = 600;

fn row(cells: &[&str]) -> Vec<Inline> {
    cells.iter().map(|c| Inline::plain(*c)).collect()
}

fn table(align: &[Align], body: &[&str]) -> Vec<Block> {
    vec![Block::Table {
        align: align.to_vec(),
        // Same length headers, so the three columns have the same share of the width.
        rows: vec![row(&["HHHHHHHHHH", "HHHHHHHHHH", "HHHHHHHHHH"]), row(body)],
    }]
}

fn dark(p: image::Rgba<u8>) -> bool {
    p[0] < 140 && p[1] < 140 && p[2] < 140
}

/// The first and last dark column in the rows `y0..y1` between `x0` and `x1`.
fn ink_x(img: &image::RgbaImage, (x0, x1): (u32, u32), (y0, y1): (u32, u32)) -> Option<(u32, u32)> {
    let cols: Vec<u32> = (x0..x1)
        .filter(|&x| (y0..y1).any(|y| dark(*img.get_pixel(x, y))))
        .collect();
    Some((*cols.first()?, *cols.last()?))
}

/// The rows of the body row: below the header row (the last text rows).
fn body_rows(img: &image::RgbaImage) -> (u32, u32) {
    let rows: Vec<u32> = (0..img.height())
        .filter(|&y| (0..W).any(|x| dark(*img.get_pixel(x, y))))
        .collect();
    let last = *rows.last().expect("ink");
    let mut first = last;
    while rows.contains(&(first - 1)) {
        first -= 1;
    }
    (first, last + 1)
}

#[test]
fn columns_are_placed_by_their_alignment() {
    let _g = serial();
    if !has_fonts() {
        eprintln!("skipped: no fonts");
        return;
    }
    let img = draw(
        TextView::new(table(
            &[Align::Center, Align::Left, Align::Right],
            &["ii", "ii", "ii"],
        ))
        .build(),
        W,
        200,
        "tabelle-ausrichtung.png",
    );
    let rows = body_rows(&img);
    let third = W / 3;
    let cols = [(0, third), (third, 2 * third), (2 * third, W)];
    let ink: Vec<(u32, u32)> = cols
        .iter()
        .map(|&c| ink_x(&img, c, rows).expect("ink in the cell"))
        .collect();
    let mid = |c: (u32, u32)| u32::midpoint(c.0, c.1);
    let (centre, left, right) = (ink[0], ink[1], ink[2]);
    assert!(
        mid(centre).abs_diff(mid(cols[0])) < 14,
        "the centre column is centred: ink {centre:?} in {:?}",
        cols[0]
    );
    assert!(
        left.0 - cols[1].0 < 24,
        "the left column starts at its left edge: {left:?}"
    );
    assert!(
        cols[2].1 - right.1 < 24,
        "the right column ends at its right edge: {right:?}"
    );
    // Counter-test: without alignment all three start at their left edge.
    let plain = draw(
        TextView::new(table(&[], &["ii", "ii", "ii"])).build(),
        W,
        200,
        "tabelle-ohne-ausrichtung.png",
    );
    let first = ink_x(&plain, cols[0], rows).unwrap();
    assert!(first.0 - cols[0].0 < 24, "no alignment: left {first:?}");
}

#[test]
fn checkboxes_show_in_lists_and_in_table_cells() {
    let _g = serial();
    if !has_fonts() {
        eprintln!("skipped: no fonts");
        return;
    }
    let list = |a: bool, b: bool| {
        vec![Block::List {
            ordered: false,
            start: 1,
            items: vec![
                ListItem::new(vec![Block::Paragraph(Inline::plain("open"))]).task(a),
                ListItem::new(vec![Block::Paragraph(Inline::plain("done"))]).task(b),
            ],
        }]
    };
    let one = draw(
        TextView::new(list(false, true)).build(),
        W,
        120,
        "aufgaben-liste.png",
    );
    let two = draw(
        TextView::new(list(true, false)).build(),
        W,
        120,
        "aufgaben-liste-getauscht.png",
    );
    assert_ne!(
        one.as_raw(),
        two.as_raw(),
        "the state of a task changes the picture"
    );
    let cells = |a: &str, b: &str| table(&[Align::Center, Align::None, Align::None], &[a, "x", b]);
    let open = draw(
        TextView::new(cells("[ ]", "[ ]")).build(),
        W,
        200,
        "aufgaben-zelle.png",
    );
    let done = draw(
        TextView::new(cells("[x]", "[x]")).build(),
        W,
        200,
        "aufgaben-zelle-erledigt.png",
    );
    let text = draw(
        TextView::new(cells("[y]", "[y]")).build(),
        W,
        200,
        "zelle-text.png",
    );
    assert_ne!(open.as_raw(), done.as_raw(), "open and done cells differ");
    assert_ne!(
        open.as_raw(),
        text.as_raw(),
        "a checkbox cell is drawn as a box, not as the letters [ ]"
    );
}
