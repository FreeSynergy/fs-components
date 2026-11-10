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

// ---- third review (2026-11-10) ----

#[test]
fn third_1_a_failed_wider_reload_keeps_the_search_that_matches_the_list() {
    let (mut s, mem) = open();
    s.begin("find");
    type_in(&mut s, "text", 0, "babbage");
    assert!(s.submit());
    let calls = mem.list_calls.load(std::sync::atomic::Ordering::SeqCst);
    *mem.fail_list_call.lock().unwrap() = Some(calls + 2);
    s.begin("create");
    type_in(&mut s, "name", 0, "Grace");
    assert!(s.submit());
    assert!(
        s.search_active(),
        "the list is still the narrowed one, so the search is still set"
    );
    assert_eq!(s.list().rows().len(), 1);
}

#[test]
fn third_2_back_from_a_record_that_was_not_loaded_reads_the_list_again() {
    let (mut s, mem) = open();
    *mem.down_after_run.lock().unwrap() = true;
    s.begin("create");
    type_in(&mut s, "name", 0, "Grace");
    assert!(s.submit());
    // still down: back says so on the list screen
    s.back_to_list();
    assert!(
        s.messages()
            .current()
            .iter()
            .any(|m| m.code == "unavailable"),
        "told on the list"
    );
    // back up: back shows the new record
    *mem.down.lock().unwrap() = false;
    s.back_to_list();
    assert_eq!(s.list().rows().len(), 3);
    assert!(!s.is_stale());
}

#[test]
fn third_5_the_same_error_is_told_once() {
    let mem = Memory::new(vec![]);
    *mem.down.lock().unwrap() = true;
    let mut s = Session::new(contact(), mem, clock());
    s.reload();
    s.reload();
    let n = s
        .messages()
        .current()
        .iter()
        .filter(|m| m.code == "unavailable")
        .count();
    assert_eq!(n, 1);
}

#[test]
fn third_6_after_a_failed_reading_the_old_record_is_not_shown_as_current() {
    let (mut s, mem) = open();
    s.open_detail("urn:1");
    assert!(s.record().is_some());
    *mem.down.lock().unwrap() = true;
    assert!(!s.reload());
    assert!(s.record().is_none(), "old values are not shown as current");
    assert!(!s.begin("change") && !s.begin("delete"));
    *mem.down.lock().unwrap() = false;
    assert!(s.reload());
    assert!(s.record().is_some());
}

#[test]
fn third_c_a_refusal_with_nothing_for_the_form_marks_nothing_and_says_so_once() {
    let (mut s, mem) = open();
    mem.create_refusal
        .store(4, std::sync::atomic::Ordering::SeqCst);
    s.begin("create");
    type_in(&mut s, "name", 0, "Grace");
    assert!(!s.submit());
    assert!(matches!(s.screen(), Screen::Command { .. }));
    assert_eq!(
        s.form().unwrap().values("name"),
        ["Grace"],
        "what was typed stays"
    );
    assert!(
        s.form().unwrap().visible_issues().is_empty(),
        "no field is marked wrongly"
    );
    let rejected = s
        .messages()
        .current()
        .into_iter()
        .filter(|m| m.code == "rejected")
        .count();
    assert_eq!(rejected, 1);
    mem.create_refusal
        .store(2, std::sync::atomic::Ordering::SeqCst);
    assert!(!s.submit());
    let marked: Vec<String> = s
        .form()
        .unwrap()
        .visible_issues()
        .into_iter()
        .map(|i| i.field)
        .collect();
    assert_eq!(marked, ["name"], "only the field the form has");
    let rejected = s
        .messages()
        .current()
        .into_iter()
        .filter(|m| m.code == "rejected")
        .count();
    assert_eq!(rejected, 1, "and still told once, not stacked");
}
