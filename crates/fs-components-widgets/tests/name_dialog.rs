//! The name dialog: words in two languages, editing keys, Enter and Escape, the buttons, the picture.
//!
//! Tests: Model tests over the words, an in-process mouse and keyboard over the built dialog, pixel tests.

mod common;

use std::sync::{Arc, Mutex};

use blinc_layout::prelude::RenderTree;
use common::{Input, background, draw, ink, layout, serial};
use fs_components_mf2::check_pairs;
use fs_components_widgets::name_dialog::{Labels, NameDialog, Purpose, TEXTS};
use fs_components_words::{Locale, Localized};

fn de() -> Labels {
    Labels::for_locale(&Locale::parse("de").unwrap())
}

type Log = Arc<Mutex<Vec<String>>>;

fn dialog(initial: &str, log: &Log) -> NameDialog {
    let (named, left) = (Arc::clone(log), Arc::clone(log));
    NameDialog::new(&Labels::default(), Purpose::Rename, "Old")
        .id("nd")
        .initial(initial)
        .on_name(move |n| named.lock().unwrap().push(format!("name {n}")))
        .on_cancel(move || left.lock().unwrap().push("cancel".into()))
}

/// A built dialog with the keyboard in the field, the way a frontend opens it (`focus`, no click: a click would move
/// the cursor and drop the selection of the preset name).
fn open(d: &NameDialog) -> (RenderTree, Input) {
    let tree = layout(d.build(), 480.0, 260.0);
    d.focus();
    blinc_layout::widgets::text_input::process_pending_input_focus();
    (tree, Input::new())
}

fn type_in(tree: &mut RenderTree, text: &str) {
    for ch in text.chars() {
        tree.broadcast_text_input_event(ch, false, false, false, false);
    }
}

#[test]
fn every_id_is_in_german_and_english_with_the_same_variables() {
    assert_eq!(check_pairs(TEXTS), Vec::<String>::new());
    assert_eq!(de().new_page, "Neue Seite");
    assert_eq!(Labels::default().new_page, "New page");
    let sub = NameDialog::new(&de(), Purpose::SubPage, "Handbuch");
    assert_eq!(sub.title(), "Neue Unterseite von Handbuch");
}

#[test]
fn the_title_follows_the_purpose() {
    let l = Labels::default();
    assert_eq!(
        NameDialog::new(&l, Purpose::NewPage, "x").title(),
        "New page"
    );
    assert_eq!(
        NameDialog::new(&l, Purpose::SubPage, "Guide").title(),
        "New sub-page of Guide"
    );
    assert_eq!(
        NameDialog::new(&l, Purpose::Rename, "Old").title(),
        "Rename Old"
    );
}

#[test]
fn the_name_goes_in_as_an_argument_not_by_replacing_braces() {
    // A name with braces comes out as it is; a `{}` replace would have treated it as a hole.
    let l = Labels::default();
    let dialog = NameDialog::new(&l, Purpose::Rename, "a {} b {$about}");
    let title = dialog.title();
    assert!(title.contains("a {} b {$about}"), "{title}");
    assert!(title.starts_with("Rename"), "{title}");
}

#[test]
fn enter_takes_the_trimmed_name_and_escape_leaves() {
    let _g = serial();
    let log = Log::default();
    let d = dialog("Start", &log);
    let (mut tree, mut input) = open(&d);
    assert_eq!(d.text(), "Start");
    type_in(&mut tree, "  Guide ");
    assert_eq!(
        d.text(),
        "  Guide ",
        "the preset name was selected: typing replaces it"
    );
    input.key(&mut tree, 13);
    assert_eq!(*log.lock().unwrap(), ["name Guide"]);
    input.key(&mut tree, 27);
    assert_eq!(
        log.lock().unwrap().last().map(String::as_str),
        Some("cancel")
    );
}

#[test]
fn a_blank_name_is_not_taken() {
    let _g = serial();
    let log = Log::default();
    let d = dialog("   ", &log);
    let (mut tree, mut input) = open(&d);
    input.key(&mut tree, 13);
    assert!(!d.confirm());
    assert!(log.lock().unwrap().is_empty());
}

#[test]
fn the_editing_keys_work_in_the_field() {
    let _g = serial();
    let log = Log::default();
    let d = dialog("", &log);
    let (mut tree, mut input) = open(&d);
    type_in(&mut tree, "abcd");
    input.key(&mut tree, 8); // Backspace
    assert_eq!(d.text(), "abc");
    input.key(&mut tree, 36); // Home
    type_in(&mut tree, "X");
    assert_eq!(d.text(), "Xabc");
    input.key(&mut tree, 127); // Delete removes the 'a'
    assert_eq!(d.text(), "Xbc");
    input.key(&mut tree, 39); // Right
    input.key(&mut tree, 8);
    assert_eq!(d.text(), "Xc");
    input.key(&mut tree, 35); // End
    type_in(&mut tree, "!");
    assert_eq!(d.text(), "Xc!");
    input.key(&mut tree, 37); // Left
    input.key(&mut tree, 8);
    assert_eq!(d.text(), "X!");
    input.press(&mut tree, 65, (false, true, false)); // Ctrl+A selects all
    type_in(&mut tree, "Z");
    assert_eq!(d.text(), "Z");
}

#[test]
fn letters_beyond_ascii_are_kept() {
    let _g = serial();
    let log = Log::default();
    let d = dialog("", &log);
    let (mut tree, _input) = open(&d);
    type_in(&mut tree, "Übersicht ß 李雷");
    assert_eq!(d.text(), "Übersicht ß 李雷");
}

#[test]
fn the_buttons_do_what_the_keys_do() {
    let _g = serial();
    let log = Log::default();
    let d = dialog("Page", &log);
    let (mut tree, mut input) = open(&d);
    input.click(&mut tree, "nd-ok");
    input.click(&mut tree, "nd-cancel");
    assert_eq!(*log.lock().unwrap(), ["name Page", "cancel"]);
}

#[test]
fn the_picture_in_both_languages() {
    let _g = serial();
    for lang in ["en", "de"] {
        let labels = Labels::for_locale(&Locale::parse(lang).unwrap());
        let d = NameDialog::new(&labels, Purpose::SubPage, "Handbook").initial("Chapter one");
        let img = draw(d.build(), 480, 220, &format!("name-dialog-{lang}.png"));
        assert!(
            ink(&img, (0, 0, 480, 220), background(&img)) > 2500,
            "{lang}"
        );
    }
}
