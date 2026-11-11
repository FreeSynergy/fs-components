//! Review of 2026-11-10 (Opus), admin frontend: the handshake error stays with its words and "Try again", read-only
//! fields stay out of the change form, a command that is locked while its form is open does not run.
//!
//! Pattern: none (tests). No window, no graphics.
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod memory;

use std::sync::Arc;
use std::sync::atomic::Ordering;

use fs_admin_frontend::{BackendError, ManualClock, Session};
use fs_types::{BasicType, FieldDescription};
use memory::{Memory, record};

const WORDS: &str =
    "The server could not be replaced (running: 0.0.2; needed: 0.0.3 protocol 1): no start";

fn open(mem: &Arc<Memory>) -> Session {
    Session::new(
        mem.description.clone(),
        mem.clone(),
        Arc::new(ManualClock::default()),
    )
}

fn handshake(text: &str) -> BackendError {
    BackendError::Handshake(text.to_owned())
}

#[test]
fn a_handshake_error_stays_with_its_words_and_an_offer_to_try_again_never_a_blank_window() {
    let mem = Memory::new(vec![record("urn:1", &[("name", &["Ada"])])]);
    *mem.list_error.lock().unwrap() = Some(handshake(WORDS));
    let s = open(&mem);
    let all = s.messages().current();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].code, "handshake");
    assert_eq!(
        all[0].detail.as_deref(),
        Some(WORDS),
        "error.to_string() as it is"
    );
    assert!(all[0].retry, "it offers to try again");
    assert!(s.is_stale(), "the old list is not shown as current");
}

#[test]
fn try_again_connects_again_and_a_second_failure_is_told_once_with_its_new_words() {
    let mem = Memory::new(vec![record("urn:1", &[("name", &["Ada"])])]);
    *mem.list_error.lock().unwrap() = Some(handshake(WORDS));
    let mut s = open(&mem);
    *mem.reconnect_error.lock().unwrap() = Some(handshake("still no"));
    assert!(!s.retry());
    assert_eq!(mem.reconnects.load(Ordering::SeqCst), 1);
    let all = s.messages().current();
    assert_eq!(all.len(), 1, "told once, however often");
    assert_eq!(all[0].detail.as_deref(), Some("still no"));
    *mem.reconnect_error.lock().unwrap() = None;
    assert!(s.retry());
    assert_eq!(mem.reconnects.load(Ordering::SeqCst), 2);
    assert!(s.messages().current().is_empty(), "the message is solved");
    assert!(!s.is_stale());
    assert_eq!(s.list().rows().len(), 1, "the records are read again");
}

#[test]
fn a_handshake_error_of_a_command_is_told_with_its_words_too() {
    let mem = Memory::new(vec![record("urn:1", &[("name", &["Ada"])])]);
    let mut s = open(&mem);
    *mem.run_error.lock().unwrap() = Some(handshake(WORDS));
    assert!(s.begin("create"));
    s.form_mut().unwrap().set("name", 0, "New");
    assert!(!s.submit());
    assert!(
        s.messages()
            .current()
            .iter()
            .any(|m| m.code == "handshake" && m.retry)
    );
}

#[test]
fn a_read_only_field_is_not_in_the_change_form_and_never_in_the_edit() {
    let mut mem = Memory::new(vec![record(
        "urn:1",
        &[("name", &["Ada"]), ("serial", &["7"])],
    )]);
    Arc::get_mut(&mut mem).unwrap().description = memory::contact().field(
        FieldDescription::new("serial", BasicType::Text)
            .read_only()
            .label("contact.serial"),
    );
    let mut s = open(&mem);
    assert!(s.open_detail("urn:1"));
    assert!(s.begin("change"));
    let form = s.form_mut().unwrap();
    assert!(form.fields().iter().all(|f| f.name != "serial"));
    assert!(form.fields().iter().any(|f| f.name == "name"));
    form.set("name", 0, "Ada L");
    let edit = form.submit_changes().expect("no finding");
    assert!(edit.fields.iter().all(|f| f.path != ["serial"]));
    assert_eq!(edit.fields.len(), 1);
}

#[test]
fn a_command_the_core_stops_offering_while_its_form_is_open_does_not_run() {
    let mem = Memory::new(vec![record("urn:1", &[("name", &["Ada"])])]);
    let mut s = open(&mem);
    assert!(s.begin("create"));
    s.form_mut().unwrap().set("name", 0, "New");
    mem.unsupported.lock().unwrap().insert("create".into());
    s.reload(); // the core is asked again when the records are read
    assert!(!s.submit(), "locked now");
    assert_eq!(s.list().rows().len(), 1, "nothing was created");
    mem.unsupported.lock().unwrap().clear();
    s.reload();
    assert!(s.submit());
    assert_eq!(s.list().rows().len(), 2);
}

#[test]
fn try_again_is_two_halves_so_the_work_holds_no_lock_and_a_second_press_does_nothing() {
    let mem = Memory::new(vec![record("urn:1", &[("name", &["Ada"])])]);
    *mem.list_error.lock().unwrap() = Some(handshake(WORDS));
    let mut s = open(&mem);
    let job = s.retry_begin().expect("the first press starts a try");
    assert!(
        s.retry_begin().is_none(),
        "a second press while one runs does nothing"
    );
    // connecting, reading and asking the core happen here, outside the session (a worker in the window)
    let calls_before = mem.list_calls.load(Ordering::SeqCst);
    let outcome = job.run();
    assert!(
        mem.list_calls.load(Ordering::SeqCst) > calls_before,
        "the reading was done by the job"
    );
    let calls_after_job = mem.list_calls.load(Ordering::SeqCst);
    assert!(s.retry_finish(outcome));
    assert_eq!(
        mem.list_calls.load(Ordering::SeqCst),
        calls_after_job,
        "putting the answers in reads nothing under the lock"
    );
    assert_eq!(mem.reconnects.load(Ordering::SeqCst), 1);
    assert_eq!(s.list().rows().len(), 1);
    assert!(s.retry_begin().is_some(), "free again after the try");
}

#[test]
fn a_backend_that_panics_in_reconnect_is_a_failed_try_and_try_again_still_works() {
    let mem = Memory::new(vec![record("urn:1", &[("name", &["Ada"])])]);
    *mem.list_error.lock().unwrap() = Some(handshake(WORDS));
    let mut s = open(&mem);
    mem.reconnect_panics.store(true, Ordering::SeqCst);
    assert!(!s.retry(), "a failed try, not a hang");
    let told = s.messages().current();
    assert!(told.iter().any(|m| m.code == "unavailable"), "{told:?}");
    mem.reconnect_panics.store(false, Ordering::SeqCst);
    assert!(
        s.retry_begin().is_some(),
        "Try again is not dead: `retrying` was reset"
    );
}
