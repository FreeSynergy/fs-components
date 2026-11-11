//! The e-mail field of the people form: every character typed reaches the draft, the dot among them
//! ("a.b@c.de" was shown as "ab@cde"). Typing is feeding the characters to the focused field the way a
//! window does, and reading the value of the draft.
//!
//! Tests: input tests over the real element, no window.

#![allow(clippy::pedantic)]

#[path = "people_common/mod.rs"]
mod people_common;

use std::sync::Arc;

use fs_components_workbenches::people::{PeopleLabels, PeopleWidget};
use people_common::Book;
use people_common::harness::Harness;

fn mail_after_typing(text: &str) -> String {
    let widget = PeopleWidget::new(Arc::new(Book::with(&["ada"])), PeopleLabels::default());
    let mut h = Harness::new(widget, 900.0, 600.0);
    h.click("edit");
    h.press("field-mail/1");
    h.type_text(text);
    h.widget.with_state(|s| {
        s.editing()
            .unwrap()
            .draft
            .fields()
            .find(|f| f.key == "mail/1")
            .unwrap()
            .value
            .clone()
    })
}

#[test]
fn a_dot_typed_into_the_email_field_stays() {
    let v = mail_after_typing("a.b@c.de");
    assert!(v.starts_with("a.b@c.de"), "value is {v:?}");
}

#[test]
fn the_other_signs_of_an_address_stay_too() {
    for text in [
        "a-b_c+d@x.io",
        "o'neil@x.io",
        "a~b^c@x.io",
        "a,b;c:d/e@x.io",
        "a(b)[c]{d}\"e\"@x.io",
    ] {
        let v = mail_after_typing(text);
        assert!(v.starts_with(text), "typed {text:?}, value is {v:?}");
    }
}
