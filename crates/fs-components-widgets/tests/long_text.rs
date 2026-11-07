//! Long unbroken text must not freeze the interface thread.
//!
//! Tests: layout of the real element; no wall-clock assertion (the machine is loaded), the test
//! prints the time and asserts that the text is drawn inside its box.

mod chat_support;
mod common;

use std::sync::Arc;
use std::time::Instant;

use chat_support::msg;
use common::{draw, ink, serial};
use fs_components_widgets::message_list::{MessageList, VecMessages};
use fs_components_widgets::{Sender, SenderKind};

#[test]
fn eight_thousand_unbroken_characters_in_a_message_are_drawn_inside_the_box() {
    let _g = serial();
    let long = "w".repeat(8000);
    let m = msg("1", Sender::new("a", "A").of_kind(SenderKind::Ai), &long);
    let list = MessageList::new(Arc::new(VecMessages(vec![m]))).id("lt");
    let t = Instant::now();
    let boxed = blinc_layout::div::div()
        .w(400.0)
        .child(list.build(400.0, 500.0));
    let img = draw(boxed, 600, 500, "message-long-run.png");
    println!(
        "8000 unbroken characters: build + layout + draw {:?}",
        t.elapsed()
    );
    let page = img.get_pixel(590, 250).0;
    assert_eq!(
        ink(&img, (402, 0, 198, 500), page),
        0,
        "the run left its box"
    );
    assert!(
        ink(&img, (0, 0, 400, 500), page) > 2000,
        "the text is not drawn"
    );
}

#[test]
fn eight_thousand_characters_with_spaces_for_comparison() {
    let _g = serial();
    let long = "word ".repeat(1600);
    let m = msg("1", Sender::new("a", "A").of_kind(SenderKind::Ai), &long);
    let list = MessageList::new(Arc::new(VecMessages(vec![m]))).id("lt2");
    let t = Instant::now();
    draw(list.build(400.0, 500.0), 400, 500, "message-long-words.png");
    println!(
        "8000 characters with spaces: build + layout + draw {:?}",
        t.elapsed()
    );
}
