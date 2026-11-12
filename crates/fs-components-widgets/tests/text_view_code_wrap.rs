//! A long code line never wraps, in every profile and every place a text view is used: it stays on
//! one line inside a box of the block's own width and scrolls sideways there.
//!
//! Tests: the real element, drawn (skipped without fonts). Text is the dark ink; the scrollbar and
//! the ground of the block are light. What a picture cannot show: the scrollbar itself and the
//! scrolling by hand.

mod common;

use std::sync::Arc;

use common::{draw, has_fonts, serial};
use fs_components_widgets::message_list::{ChatMessage, MessageList, VecMessages};
use fs_components_widgets::text::Block;
use fs_components_widgets::{Sender, TextView};

/// The block is this wide inside a picture of `PICTURE`.
const BOX: u32 = 300;
const BOX_PX: f32 = 300.0;
const PICTURE: u32 = 700;
/// A message's block ends inside the list, left of the row's padding; the list itself clips at `BOX`,
/// so a block that runs out of its message would end at the list's edge instead.
const CHAT_EDGE: u32 = BOX - 20;

fn long_code() -> Vec<Block> {
    vec![Block::Code {
        lang: String::new(),
        text: "word ".repeat(80), // 400 characters
    }]
}

fn dark(p: image::Rgba<u8>) -> bool {
    p[0] < 120 && p[1] < 120 && p[2] < 120
}

/// Rows (min, max) with text in columns `x0..x1`.
fn text_rows(img: &image::RgbaImage, x0: u32, x1: u32) -> Option<(u32, u32)> {
    let rows: Vec<u32> = (0..img.height())
        .filter(|&y| (x0..x1.min(img.width())).any(|x| dark(*img.get_pixel(x, y))))
        .collect();
    Some((*rows.first()?, *rows.last()?))
}

/// Columns of text in `rows`, the last one.
fn last_text_column(img: &image::RgbaImage, rows: (u32, u32)) -> Option<u32> {
    (0..img.width())
        .rev()
        .find(|&x| (rows.0..=rows.1).any(|y| dark(*img.get_pixel(x, y))))
}

/// (a) one line, (b) the line is cut at the box, which only a box of the block's width does,
/// (c) nothing right of the box.
fn assert_one_line_in_its_box(img: &image::RgbaImage, what: &str, edge: u32) {
    let rows = text_rows(img, 0, BOX).unwrap_or_else(|| panic!("{what}: no code drawn"));
    assert!(
        rows.1 - rows.0 < 22,
        "{what}: the line wrapped (text over rows {rows:?})"
    );
    let right = last_text_column(img, rows).expect("text");
    assert!(
        right < edge,
        "{what}: text runs out of its block (to column {right}, block ends at {edge})"
    );
    assert!(
        right > edge - 60,
        "{what}: the line is not cut at the block's edge (ends at {right})"
    );
}

fn in_a_box(view: &TextView) -> blinc_layout::div::Div {
    blinc_layout::div::div().w(BOX_PX).child(view.build())
}

#[test]
fn the_standard_profile_keeps_a_long_code_line_on_one_line_inside_its_block() {
    let _g = serial();
    if !has_fonts() {
        return;
    }
    let img = draw(
        in_a_box(&TextView::new(long_code())),
        PICTURE,
        400,
        "code-wrap-standard.png",
    );
    assert_one_line_in_its_box(&img, "standard profile", BOX);
}

#[test]
fn the_reading_profile_keeps_a_long_code_line_on_one_line_inside_its_block() {
    let _g = serial();
    if !has_fonts() {
        return;
    }
    let img = draw(
        in_a_box(&TextView::new(long_code()).reading()),
        PICTURE,
        400,
        "code-wrap-reading.png",
    );
    assert_one_line_in_its_box(&img, "reading profile", BOX);
}

#[test]
fn a_chat_message_keeps_a_long_code_line_on_one_line_inside_its_list() {
    let _g = serial();
    if !has_fonts() {
        return;
    }
    let message = ChatMessage::new("m", Sender::new("p", "Sam"), long_code());
    let list = MessageList::new(Arc::new(VecMessages(vec![message]))).id("wrap");
    let img = draw(
        list.build(BOX_PX, 300.0),
        PICTURE,
        300,
        "code-wrap-chat.png",
    );
    // the name and the time are dark text too: look below the header
    let below = image::imageops::crop_imm(&img, 0, 40, PICTURE, 260).to_image();
    assert_one_line_in_its_box(&below, "message list", CHAT_EDGE);
}
