//! Tests for the findings of the independent review (2026-11-10): the session.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod common;

use common::{Memory, clock, contact, record};
use fs_admin_frontend::{Screen, Session};

fn open() -> (Session, std::sync::Arc<Memory>) {
    let mem = Memory::new(vec![
        record(
            "urn:1",
            &[
                ("name", &["Ada Lovelace"]),
                ("emails", &["ada@example.org"]),
            ],
        ),
        record("urn:2", &[("name", &["Charles Babbage"])]),
    ]);
    (Session::new(contact(), mem.clone(), clock()), mem)
}

fn type_in(s: &mut Session, field: &str, i: usize, text: &str) {
    assert!(s.form_mut().unwrap().set(field, i, text));
}

// ---- findings of the independent review (2026-11-10) ----

#[test]
fn review_2_a_created_record_is_shown_even_when_the_reload_fails_and_the_error_stays() {
    let (mut s, mem) = open();
    *mem.down_after_run.lock().unwrap() = true;
    s.begin("create");
    type_in(&mut s, "name", 0, "Grace");
    assert!(s.submit());
    assert!(matches!(s.screen(), Screen::Detail(_)), "{:?}", s.screen());
    let codes: Vec<String> = s.messages().current().into_iter().map(|m| m.code).collect();
    assert!(
        codes.contains(&"unavailable".to_owned()),
        "the failed reload is not silent: {codes:?}"
    );
    assert!(codes.contains(&"done".to_owned()));
}

#[test]
fn review_3_a_refusal_on_the_confirm_screen_is_an_error_message() {
    let (mut s, mem) = open();
    *mem.reject_delete.lock().unwrap() = true;
    s.open_detail("urn:1");
    s.begin("delete");
    assert!(!s.submit());
    let codes: Vec<String> = s.messages().current().into_iter().map(|m| m.code).collect();
    assert_eq!(codes, ["rejected"], "never silent");
    assert_eq!(mem.records.lock().unwrap().len(), 2);
}

#[test]
fn review_5_an_emptied_value_is_sent_as_cleared_and_unchanged_fields_are_not_sent() {
    let (mut s, mem) = open();
    s.open_detail("urn:1");
    s.begin("change");
    type_in(&mut s, "emails", 0, "");
    assert!(s.submit());
    let stored = mem.records.lock().unwrap();
    assert!(
        !stored[0].values.contains_key("emails"),
        "cleared: {:?}",
        stored[0].values
    );
    assert_eq!(stored[0].values["name"], ["Ada Lovelace"]);
}

#[test]
fn review_10_a_refused_search_marks_the_search_form() {
    let (mut s, mem) = open();
    *mem.reject_search.lock().unwrap() = true;
    s.begin("find");
    type_in(&mut s, "text", 0, "x");
    assert!(!s.submit());
    assert!(matches!(s.screen(), Screen::Command { .. }));
    assert_eq!(
        s.form().unwrap().visible_issues().len(),
        1,
        "the findings reach the form"
    );
}

#[test]
fn review_13_back_from_a_record_keeps_the_search() {
    let (mut s, _) = open();
    s.begin("find");
    type_in(&mut s, "text", 0, "babbage");
    assert!(s.submit());
    assert!(s.open_detail("urn:2"));
    s.back_to_list();
    assert_eq!(*s.screen(), Screen::List);
    assert_eq!(s.list().rows().len(), 1, "still narrowed");
}

#[test]
fn review_14_a_new_id_is_never_one_that_exists() {
    let (mut s, mem) = open();
    for name in ["A", "B"] {
        s.begin("create");
        type_in(&mut s, "name", 0, name);
        assert!(s.submit());
        s.open_list();
        s.open_detail(&mem.records.lock().unwrap().last().unwrap().id.clone());
        s.begin("delete");
        s.submit();
    }
    s.begin("create");
    type_in(&mut s, "name", 0, "C");
    s.submit();
    let ids: Vec<String> = mem
        .records
        .lock()
        .unwrap()
        .iter()
        .map(|r| r.id.clone())
        .collect();
    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(ids.len(), sorted.len());
}
