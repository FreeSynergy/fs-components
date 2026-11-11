//! A command in halves (v0.0.10): `submit_begin`, `SubmitJob::run`, `submit_finish`. No window, no graphics.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod memory;

use fs_admin_frontend::{Screen, Session};
use memory::{Memory, clock, contact, record};

#[test]
fn a_second_press_sends_nothing_and_the_screen_stays_until_the_answer() {
    let mem = Memory::new(vec![record("urn:1", &[("name", &["Ada"])])]);
    let mut s = Session::new(contact(), mem.clone(), clock());
    assert!(s.begin("create"));
    s.form_mut().unwrap().set("name", 0, "Grace");
    let job = s
        .submit_begin()
        .expect("the first press starts the command");
    assert!(s.is_busy());
    assert!(s.submit_begin().is_none(), "a second press sends nothing");
    assert!(!s.begin("create"), "no other command opens meanwhile");
    s.cancel();
    assert!(matches!(s.screen(), Screen::Command { .. }));
    let outcome = job.run();
    assert!(s.submit_finish(outcome));
    assert!(!s.is_busy());
    assert!(matches!(s.screen(), Screen::Detail(_)));
    assert_eq!(mem.records.lock().unwrap().len(), 2);
}

#[test]
fn a_refused_command_frees_the_session() {
    let mem = Memory::new(vec![]);
    let mut s = Session::new(contact(), mem.clone(), clock());
    assert!(s.begin("create"));
    s.form_mut().unwrap().set("name", 0, "Grace");
    *mem.run_error.lock().unwrap() = Some(fs_admin_frontend::BackendError::Unavailable(
        "timed out".into(),
    ));
    assert!(!s.submit());
    assert!(!s.is_busy());
    assert!(
        s.messages()
            .current()
            .iter()
            .any(|m| m.code == "unavailable")
    );
}
