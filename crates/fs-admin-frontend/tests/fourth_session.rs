//! Tests for the findings of the fourth review (2026-11-10): stale records, refusals, slot mapping.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod common;

use common::{Memory, clock, contact, record, values};
use fs_admin_frontend::{FormModel, Session};
use fs_types::{RecordIssue, RecordProblem, TypeProblem};

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

fn codes(s: &Session) -> Vec<String> {
    s.messages().current().into_iter().map(|m| m.code).collect()
}

#[test]
fn fourth_1_a_search_that_works_ends_the_stale_state_and_its_error() {
    let (mut s, mem) = open();
    *mem.down.lock().unwrap() = true;
    assert!(!s.reload());
    assert!(s.is_stale());
    *mem.down.lock().unwrap() = false;
    s.begin("find");
    type_in(&mut s, "term", 0, "ada");
    assert!(s.submit());
    assert!(!s.is_stale());
    assert!(!codes(&s).contains(&"unavailable".to_owned()));
    assert!(s.open_detail("urn:1"));
    assert!(
        s.record().is_some(),
        "the record of a working reading is shown"
    );
}

#[test]
fn fourth_2_leaving_a_form_while_the_records_are_old_reads_again_and_says_so() {
    let (mut s, mem) = open();
    *mem.down.lock().unwrap() = true;
    s.reload();
    s.begin("create");
    s.cancel();
    assert!(
        codes(&s).contains(&"unavailable".to_owned()),
        "told again on the list: {:?}",
        codes(&s)
    );
    *mem.down.lock().unwrap() = false;
    s.begin("create");
    s.cancel();
    assert!(!s.is_stale());
    assert_eq!(s.list().rows().len(), 2);
    assert!(codes(&s).is_empty());
}

#[test]
fn fourth_3_a_later_refusal_without_a_mark_for_the_form_clears_the_old_marks() {
    let (mut s, mem) = open();
    s.begin("create");
    type_in(&mut s, "name", 0, "Grace");
    type_in(&mut s, "emails", 0, "g@navy.test");
    mem.create_refusal
        .store(3, std::sync::atomic::Ordering::SeqCst);
    assert!(!s.submit());
    assert_eq!(s.form().unwrap().visible_issues().len(), 1);
    mem.create_refusal
        .store(4, std::sync::atomic::Ordering::SeqCst);
    assert!(!s.submit());
    assert!(
        s.form().unwrap().visible_issues().is_empty(),
        "the mark of the earlier try is gone"
    );
    assert!(codes(&s).contains(&"rejected".to_owned()));
}

#[test]
fn fourth_4_after_the_wider_reading_failed_the_list_is_the_fresh_narrow_one_not_stale() {
    let (mut s, mem) = open();
    s.begin("find");
    type_in(&mut s, "term", 0, "babbage");
    assert!(s.submit());
    let calls = mem.list_calls.load(std::sync::atomic::Ordering::SeqCst);
    *mem.fail_list_call.lock().unwrap() = Some(calls + 2);
    s.begin("create");
    type_in(&mut s, "name", 0, "Grace");
    assert!(s.submit());
    assert!(s.search_active());
    assert!(!s.is_stale(), "the narrow reading worked");
    assert!(
        codes(&s).contains(&"unavailable".to_owned()),
        "and the failed one is still told"
    );
}

#[test]
fn fourth_5_a_finding_past_the_last_sent_value_points_at_the_last_sent_slot() {
    let fields = contact().find_command("create").unwrap().input.clone();
    let mut form = FormModel::new(&fields, &values(&[]));
    form.set("name", 0, "x");
    form.add_value("emails");
    form.add_value("emails");
    form.set("emails", 0, "a@b.test");
    form.set("emails", 1, "c@d.test"); // slot 2 stays empty and is not sent
    assert!(form.submit().is_ok());
    form.refuse(vec![RecordIssue {
        field: "emails".into(),
        index: 5,
        problem: RecordProblem::Value(TypeProblem::Character),
    }]);
    let shown = form.visible_issues();
    assert_eq!(shown.len(), 1);
    assert_eq!(
        shown[0].index, 1,
        "the last value that was sent, in its slot"
    );
}
