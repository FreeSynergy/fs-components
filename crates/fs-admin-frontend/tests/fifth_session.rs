//! Tests for the findings of the fifth review (2026-11-10): opening a record while the records are old, stored searches.
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

fn codes(s: &Session) -> Vec<String> {
    s.messages().current().into_iter().map(|m| m.code).collect()
}

#[test]
fn fifth_2_a_record_opened_while_the_program_is_down_says_so() {
    let (mut s, mem) = open();
    *mem.down.lock().unwrap() = true;
    assert!(!s.reload());
    assert!(s.open_detail("urn:1"), "the list still knows the id");
    assert_eq!(*s.screen(), Screen::Detail("urn:1".into()));
    assert!(s.record().is_none(), "old values are not shown as current");
    assert!(codes(&s).contains(&"unavailable".to_owned()));
}

#[test]
fn fifth_2_a_record_opened_when_the_program_is_back_is_read_again_and_shown() {
    let (mut s, mem) = open();
    *mem.down.lock().unwrap() = true;
    s.reload();
    *mem.down.lock().unwrap() = false;
    mem.records.lock().unwrap()[0]
        .values
        .insert("name".into(), vec!["Ada King".into()]);
    assert!(s.open_detail("urn:1"));
    assert!(!s.is_stale());
    assert_eq!(
        s.record().unwrap().values["name"],
        ["Ada King"],
        "the new reading, not the old list"
    );
}

#[test]
fn fifth_1_a_record_that_is_gone_after_reading_again_leads_to_the_list_with_not_found() {
    let (mut s, mem) = open();
    *mem.down.lock().unwrap() = true;
    s.reload();
    *mem.down.lock().unwrap() = false;
    mem.records.lock().unwrap().retain(|r| r.id != "urn:1");
    assert!(!s.open_detail("urn:1"));
    assert_eq!(*s.screen(), Screen::List);
    assert_eq!(s.list().selected(), None);
    assert!(
        codes(&s).contains(&"not-found".to_owned()),
        "{:?}",
        codes(&s)
    );
}

#[test]
fn fifth_1_cancel_back_to_a_record_that_is_gone_does_the_same() {
    let (mut s, mem) = open();
    assert!(s.open_detail("urn:2"));
    assert!(s.begin("change"));
    *mem.down.lock().unwrap() = true;
    s.reload();
    *mem.down.lock().unwrap() = false;
    mem.records.lock().unwrap().retain(|r| r.id != "urn:2");
    s.cancel();
    assert_eq!(*s.screen(), Screen::List);
    assert!(codes(&s).contains(&"not-found".to_owned()));
}

#[test]
fn fifth_3_a_stored_search_that_is_refused_later_is_dropped_and_does_not_make_the_records_old() {
    let (mut s, mem) = open();
    s.begin("find");
    s.form_mut().unwrap().set("text", 0, "babbage");
    assert!(s.submit());
    assert!(s.search_active());
    *mem.reject_search.lock().unwrap() = true;
    assert!(s.reload(), "after dropping the search everything is read");
    assert!(!s.search_active());
    assert!(!s.is_stale());
    assert_eq!(s.list().rows().len(), 2);
    assert!(
        codes(&s).contains(&"rejected".to_owned()),
        "and the person is told"
    );
    let before = mem.list_calls.load(std::sync::atomic::Ordering::SeqCst);
    s.back_to_list();
    assert_eq!(
        mem.list_calls.load(std::sync::atomic::Ordering::SeqCst),
        before,
        "no repeated refusal on the next screen"
    );
}

#[test]
fn fifth_3_only_an_unreachable_program_makes_the_records_old() {
    let (mut s, mem) = open();
    *mem.list_error.lock().unwrap() = Some(fs_admin_frontend::BackendError::NotFound);
    assert!(!s.reload());
    assert!(
        !s.is_stale(),
        "a missing thing is not an unreachable program"
    );
    *mem.list_error.lock().unwrap() =
        Some(fs_admin_frontend::BackendError::Unavailable("x".into()));
    assert!(!s.reload());
    assert!(s.is_stale());
}
