//! The page title field: words in two languages, typing, Enter hands the keyboard on.
//!
//! Tests: Model tests over the words, an in-process keyboard over the built field. The look (size, no frame) is a
//! picture test of the Notion frontend (`fs-components-workbenches/tests/notion_title.rs`), where fonts are checked.

mod common;

use std::sync::{Arc, Mutex};

use blinc_layout::prelude::RenderTree;
use common::{Input, layout, serial};
use fs_components_mf2::check_pairs;
use fs_components_widgets::page_title::{PageTitle, TEXTS, TitleLabels};
use fs_components_words::{Locale, Localized};

fn de() -> TitleLabels {
    TitleLabels::for_locale(&Locale::parse("de").unwrap())
}

fn open(title: &PageTitle) -> RenderTree {
    let tree = layout(title.build(), 800.0, 120.0);
    title.focus();
    blinc_layout::widgets::text_input::process_pending_input_focus();
    tree
}

#[test]
fn the_placeholder_is_in_german_and_english() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
    assert_eq!(TitleLabels::default().placeholder, "Untitled");
    assert_eq!(de().placeholder, "Ohne Titel");
}

#[test]
fn typing_changes_the_title_and_reports_it() {
    let _serial = serial();
    let seen = Arc::new(Mutex::new(Vec::<String>::new()));
    let log = Arc::clone(&seen);
    let title = PageTitle::new(&TitleLabels::default(), "Hand")
        .on_change(move |t| log.lock().unwrap().push(t.to_owned()));
    let mut tree = open(&title);
    // The title is shown with the cursor at its end, nothing selected: typing adds.
    for ch in "book".chars() {
        tree.broadcast_text_input_event(ch, false, false, false, false);
    }
    assert_eq!(title.text(), "Handbook");
    assert_eq!(
        seen.lock().unwrap().last().map(String::as_str),
        Some("Handbook")
    );
}

#[test]
fn enter_hands_the_keyboard_on_and_adds_no_line() {
    let _serial = serial();
    let entered = Arc::new(Mutex::new(0));
    let count = Arc::clone(&entered);
    let title = PageTitle::new(&TitleLabels::default(), "Handbook")
        .on_enter(move || *count.lock().unwrap() += 1);
    let mut tree = open(&title);
    let mut input = Input::new();
    input.key(&mut tree, 13);
    assert_eq!(*entered.lock().unwrap(), 1, "Enter in the title");
    assert_eq!(title.text(), "Handbook", "no newline in the title");
    // Counter-test: without the keyboard in the title, Enter does not reach the handler.
    blinc_layout::widgets::text_input::blur_all_text_inputs();
    input.key(&mut tree, 13);
    assert_eq!(
        *entered.lock().unwrap(),
        1,
        "Enter elsewhere is not the title's"
    );
}
