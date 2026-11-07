//! A consumer can tell a click on a row from a key, and can name the content type.
//!
//! Tests: in-process input over the window's router; broadcast guard checked with two lists.

mod common;

use std::sync::{Arc, Mutex};

use common::{Input, layout, serial};
use fs_components_widgets::SortableList;
use fs_components_widgets::keys::Key;
use fs_components_widgets::sortable_list::{
    Bindings, Chosen, Command, Content, Item, Lane, Outcome, Source,
};

fn board(id: &str) -> SortableList {
    SortableList::new(vec![Lane::new(
        "l",
        "L",
        ["a", "b", "c"]
            .iter()
            .map(|k| Item::new(*k, k.to_uppercase()))
            .collect(),
    )])
    .id(id)
}

fn watch(list: SortableList) -> (SortableList, Arc<Mutex<Vec<Outcome>>>) {
    let got: Arc<Mutex<Vec<Outcome>>> = Arc::default();
    let sink = Arc::clone(&got);
    (
        list.on_outcome(Arc::new(move |o| sink.lock().unwrap().push(o.clone()))),
        got,
    )
}

#[test]
fn the_content_type_can_be_named_by_a_consumer() {
    let draw: Content = Arc::new(|item| Box::new(blinc_layout::prelude::text(item.label.clone())));
    let _ = board("ct").content(draw);
}

#[test]
fn a_click_says_it_was_a_click() {
    let _g = serial();
    let (list, got) = watch(board("sc"));
    let mut tree = layout(list.build(), 400.0, 300.0);
    Input::new().click(&mut tree, "sc-l-b");
    assert_eq!(
        got.lock().unwrap().last(),
        Some(&Outcome::Chosen(Chosen::new("b", Source::Pointer)))
    );
    assert_eq!(
        list.cursor_key().as_deref(),
        Some("b"),
        "a click still puts the cursor there"
    );
}

#[test]
fn a_key_says_it_was_a_key_and_only_the_focused_list_speaks() {
    let _g = serial();
    // Activate (Enter or Space) opens instead of picking up
    let keys = || Bindings::default().bind(Key::Activate, Command::Open);
    let (a, got_a) = watch(board("ka").keys(keys()));
    let (b, got_b) = watch(board("kb").keys(keys()));
    let ui = blinc_layout::div::div()
        .flex_col()
        .child(a.build())
        .child(b.build());
    let mut tree = layout(ui, 400.0, 600.0);
    let mut input = Input::new();
    input.focus(&tree, "ka");
    input.key(&mut tree, 40); // Down: cursor to b
    input.key(&mut tree, 13); // Enter
    assert_eq!(
        got_a.lock().unwrap().last(),
        Some(&Outcome::Chosen(Chosen::new("b", Source::Keyboard)))
    );
    assert!(
        got_b.lock().unwrap().is_empty(),
        "the other list heard nothing"
    );
    assert_eq!(a.grabbed(), None, "Open does not pick the card up");
}

#[test]
fn open_with_nothing_to_open_is_refused() {
    let _g = serial();
    let empty = SortableList::new(vec![]);
    assert!(matches!(empty.command(Command::Open), Outcome::Refused(_)));
}
