//! Typing into the e-mail and phone fields: every character of an address reaches the value, the dot
//! among them ("a.b@c.de" was shown as "ab@cde"). Typing here is feeding the characters to the field the
//! way the window does (`broadcast_text_input_event`) and reading the value.
//!
//! Pattern: none (tests). They need a graphics adapter.
#![allow(clippy::pedantic)]

#[path = "../../fs-components-widgets/tests/common/mod.rs"]
mod common;

use std::sync::Arc;

use common::{Input, frame, layout, serial};
use fs_components_fields::{FieldInput, KeysAsText, Kind, Look};
use fs_components_styles::StyleId;

fn typed(kind: Kind, first_id: &str, text: &str) -> FieldInput {
    let field =
        FieldInput::new(kind, Look::new(StyleId::Admin, false), Arc::new(KeysAsText)).name("m");
    let mut tree = layout(field.build(), 600.0, 200.0);
    let mut input = Input::new();
    input.click(&mut tree, first_id);
    for ch in text.chars() {
        tree.broadcast_text_input_event(ch, false, false, false, false);
        frame(&mut tree, 600.0, 200.0);
    }
    field
}

#[test]
fn every_character_of_an_address_arrives_the_dot_too() {
    let _g = serial();
    for text in ["a.b@c.de", "ada.lovelace@mail.example.org", "a-b_c+d@x.io"] {
        let field = typed(Kind::Email, "m-local", text);
        assert_eq!(
            field.text(),
            text,
            "typed {text:?}, parts {:?}",
            field.parts()
        );
    }
}

#[test]
fn a_dot_typed_into_the_first_part_stays() {
    let _g = serial();
    let field = typed(Kind::Email, "m-local", "a.b");
    assert_eq!(field.parts()[0], "a.b");
}

#[test]
fn the_special_signs_of_a_phone_number_arrive() {
    let _g = serial();
    let field = typed(Kind::Phone, "m-country", "+49");
    assert!(
        field.parts().concat().contains("+49"),
        "{:?}",
        field.parts()
    );
}
