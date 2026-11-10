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
    assert!(
        s.record().is_none(),
        "the new record could not be read, it is not pretended"
    );
    assert!(
        !s.begin("change") && !s.begin("delete"),
        "and nothing is done to a record that is not loaded"
    );
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

#[test]
fn review_a_a_failed_reload_never_lets_a_command_hit_the_record_selected_before() {
    let (mut s, mem) = open();
    s.open_detail("urn:1");
    s.back_to_list();
    *mem.down_after_run.lock().unwrap() = true;
    s.begin("create");
    type_in(&mut s, "name", 0, "Grace");
    assert!(s.submit());
    assert_eq!(s.list().selected(), None, "the old selection is gone");
    let started = s.begin("delete");
    let aimed_at_old = matches!(s.screen(), Screen::Confirm { record, .. } if record == "urn:1");
    assert!(
        !aimed_at_old,
        "urn:1 must not be what the question is about"
    );
    assert!(!started);
    assert_eq!(mem.records.lock().unwrap().len(), 3, "nothing was removed");
}

#[test]
fn review_a_a_command_on_a_record_uses_the_record_of_the_detail_screen() {
    let (mut s, mem) = open();
    s.open_detail("urn:2");
    s.list_mut().select("urn:1"); // the selection moved elsewhere
    assert!(s.begin("delete"));
    assert!(matches!(s.screen(), Screen::Confirm { record, .. } if record == "urn:2"));
    assert!(s.submit());
    let ids: Vec<String> = mem
        .records
        .lock()
        .unwrap()
        .iter()
        .map(|r| r.id.clone())
        .collect();
    assert_eq!(ids, ["urn:1"]);
}

#[test]
fn review_b_a_failed_second_reload_is_reported_on_the_screen_that_stays() {
    let (mut s, mem) = open();
    s.begin("find");
    type_in(&mut s, "text", 0, "babbage");
    assert!(s.submit());
    let calls = mem.list_calls.load(std::sync::atomic::Ordering::SeqCst);
    // create: the first reload (narrowed list, the new record is not in it) works, the second (all) is lost
    *mem.fail_list_call.lock().unwrap() = Some(calls + 2);
    s.begin("create");
    type_in(&mut s, "name", 0, "Grace");
    assert!(s.submit());
    let codes: Vec<String> = s.messages().current().into_iter().map(|m| m.code).collect();
    assert!(
        codes.contains(&"unavailable".to_owned()),
        "{codes:?} on {:?}",
        s.screen()
    );
}

#[test]
fn review_c_a_refusal_with_no_findings_is_a_message_and_a_partial_one_marks_what_it_can() {
    let (mut s, mem) = open();
    mem.create_refusal
        .store(1, std::sync::atomic::Ordering::SeqCst);
    s.begin("create");
    type_in(&mut s, "name", 0, "Grace");
    assert!(!s.submit());
    let codes: Vec<String> = s.messages().current().into_iter().map(|m| m.code).collect();
    assert_eq!(codes, ["rejected"], "not silent");
    mem.create_refusal
        .store(2, std::sync::atomic::Ordering::SeqCst);
    assert!(!s.submit());
    assert_eq!(
        s.form().unwrap().visible_issues().len(),
        1,
        "the finding on name is marked"
    );
    assert!(
        s.messages().current().iter().any(|m| m.code == "rejected"),
        "the one on an unknown field is told"
    );
}

#[test]
fn review_d_a_refusal_counted_over_the_sent_values_marks_the_slot_it_means() {
    let (mut s, mem) = open();
    mem.create_refusal
        .store(3, std::sync::atomic::Ordering::SeqCst);
    s.begin("create");
    type_in(&mut s, "name", 0, "Grace");
    assert!(s.form_mut().unwrap().add_value("emails"));
    type_in(&mut s, "emails", 1, "g@navy.test"); // slot 0 stays empty, so the program sees ONE value, its index 0
    assert!(!s.submit());
    let shown = s.form().unwrap().visible_issues();
    assert_eq!(shown.len(), 1);
    assert_eq!(
        (shown[0].field.as_str(), shown[0].index),
        ("emails", 1),
        "the typed slot, not the empty one"
    );
}

#[test]
fn review_h_only_a_real_difference_counts_as_unsaved_and_a_search_form_never_does() {
    let (mut s, _) = open();
    s.open_detail("urn:1");
    s.begin("change");
    assert!(!s.has_unsaved_form());
    type_in(&mut s, "name", 0, "  Ada Lovelace  ");
    assert!(!s.has_unsaved_form(), "the same value, only spaces");
    type_in(&mut s, "name", 0, "Ada King");
    assert!(s.has_unsaved_form());
    s.cancel();
    s.begin("find");
    type_in(&mut s, "text", 0, "ada");
    assert!(!s.has_unsaved_form(), "a search is not work to lose");
}
