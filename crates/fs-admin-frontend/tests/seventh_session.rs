//! Tests of `change` as an `Edit` (fs-interfaces v0.0.11): typed field by field, no JSON, the id in the call.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod common;

use common::{Memory, clock, contact, record};
use fs_admin_frontend::{Screen, Session};
use fs_types::BasicType;

fn open() -> (Session, std::sync::Arc<Memory>) {
    let mem = Memory::new(vec![record(
        "urn:1",
        &[
            ("name", &["Ada"]),
            ("emails", &["ada@example.org"]),
            ("phone", &["+442071234567"]),
        ],
    )]);
    (Session::new(contact(), mem.clone(), clock()), mem)
}

#[test]
fn change_sends_an_edit_with_only_the_changed_fields_typed_and_the_id_in_the_call() {
    let (mut s, mem) = open();
    assert!(s.open_detail("urn:1"));
    assert!(s.begin("change"));
    let fields: Vec<String> = s
        .form()
        .unwrap()
        .fields()
        .iter()
        .map(|f| f.name.clone())
        .collect();
    assert_eq!(
        fields,
        ["name", "emails", "phone", "note"],
        "the form is the fields of the type"
    );
    s.form_mut().unwrap().set("name", 0, "Ada King");
    s.form_mut().unwrap().set("phone", 0, "");
    assert!(s.submit());
    let edit = mem
        .last_edit
        .lock()
        .unwrap()
        .clone()
        .expect("the program got an Edit");
    let paths: Vec<_> = edit.fields.iter().map(|f| f.path.clone()).collect();
    assert_eq!(
        paths,
        [vec!["name".to_owned()], vec!["phone".to_owned()]],
        "emails was not touched"
    );
    assert_eq!(edit.fields[0].basic, BasicType::Text);
    assert_eq!(edit.fields[0].values, ["Ada King"]);
    assert_eq!(edit.fields[1].basic, BasicType::Phone);
    assert!(edit.fields[1].values.is_empty(), "cleared = no values");
    let stored = mem.records.lock().unwrap();
    assert_eq!(stored[0].values["name"], ["Ada King"]);
    assert!(!stored[0].values.contains_key("phone"));
    assert_eq!(
        stored[0].values["emails"],
        ["ada@example.org"],
        "untouched stays"
    );
    drop(stored);
    assert_eq!(*s.screen(), Screen::Detail("urn:1".into()));
}

#[test]
fn a_change_the_rules_refuse_is_not_sent() {
    let (mut s, mem) = open();
    s.open_detail("urn:1");
    s.begin("change");
    s.form_mut().unwrap().set("emails", 0, "not an address");
    assert!(!s.submit());
    assert!(
        mem.last_edit.lock().unwrap().is_none(),
        "the program was not asked"
    );
    assert_eq!(s.form().unwrap().visible_issues().len(), 1);
}

#[test]
fn emptying_a_required_field_is_refused_before_it_is_sent() {
    let (mut s, mem) = open();
    s.open_detail("urn:1");
    s.begin("change");
    s.form_mut().unwrap().set("name", 0, "");
    assert!(!s.submit());
    assert!(mem.last_edit.lock().unwrap().is_none());
}

#[test]
fn a_refusal_by_the_program_on_a_change_marks_the_field() {
    let (mut s, mem) = open();
    *mem.forbidden.lock().unwrap() = Some("Mallory".into());
    s.open_detail("urn:1");
    s.begin("change");
    s.form_mut().unwrap().set("name", 0, "Mallory");
    assert!(!s.submit());
    assert_eq!(s.form().unwrap().visible_issues()[0].field, "name");
}
