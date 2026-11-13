//! The name dialog: words in two languages, editing keys, Enter and Escape, the buttons, the picture.
//!
//! Tests: Model tests over the words, an in-process mouse and keyboard over the built dialog, pixel tests.

mod common;

use std::sync::{Arc, Mutex};

use blinc_layout::prelude::RenderTree;
use common::{Input, background, draw, ink, layout, serial};
use fs_components_widgets::name_dialog::{Labels, NameDialog, Purpose, texts};
use fs_components_words::{Args, Catalog, TextId, Words};

type Log = Arc<Mutex<Vec<String>>>;

fn dialog(initial: &str, log: &Log) -> NameDialog {
    let (named, left) = (Arc::clone(log), Arc::clone(log));
    NameDialog::new(&Labels::default(), Purpose::Rename, "Old")
        .id("nd")
        .initial(initial)
        .on_name(move |n| named.lock().unwrap().push(format!("name {n}")))
        .on_cancel(move || left.lock().unwrap().push("cancel".into()))
}

fn placeholders(text: &str) -> Vec<String> {
    let mut names: Vec<String> = text
        .split('{')
        .skip(1)
        .filter_map(|p| p.split('}').next().map(str::to_owned))
        .collect();
    names.sort();
    names
}

fn said(c: &Catalog, id: &str) -> String {
    c.say(&TextId::new(id), &Args::new().text("about", "<about>"))
        .text()
        .replace("<about>", "{about}")
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
fn both_languages_have_every_text_with_the_same_placeholders() {
    let (en, de) = (texts::english(), texts::german());
    assert!(en.validate().is_empty() && de.validate().is_empty());
    for id in texts::IDS {
        let (a, b) = (said(&en, id), said(&de, id));
        let args = Args::new().text("about", "x");
        assert!(en.say(&TextId::new(id), &args).is_found(), "{id} en");
        assert!(de.say(&TextId::new(id), &args).is_found(), "{id} de");
        assert_eq!(placeholders(&a), placeholders(&b), "{id}: {a} / {b}");
    }
    assert_eq!(Labels::from_words(&de).new_page, "Neue Seite");
    assert_eq!(Labels::from_words(&en).new_page, "New page");
    let sub = NameDialog::new(&Labels::from_words(&de), Purpose::SubPage, "Handbuch");
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
    for (lang, catalog) in [("en", texts::english()), ("de", texts::german())] {
        let labels = Labels::from_words(&catalog);
        let d = NameDialog::new(&labels, Purpose::SubPage, "Handbook").initial("Chapter one");
        let img = draw(d.build(), 480, 220, &format!("name-dialog-{lang}.png"));
        assert!(
            ink(&img, (0, 0, 480, 220), background(&img)) > 2500,
            "{lang}"
        );
    }
}
