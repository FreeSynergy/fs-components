//! A command in halves (v0.0.10): `submit_begin`, `SubmitJob::run`, `submit_finish`. No window, no graphics.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod memory;

use std::sync::{Arc, Mutex};
use std::thread::ThreadId;

use fs_admin_frontend::{Backend, BackendError, Outcome, Record, Screen, Session, Values};
use memory::{Memory, clock, contact, record};

/// Remembers on which threads `list` was asked.
struct Spy {
    inner: Arc<Memory>,
    lists: Mutex<Vec<ThreadId>>,
}

impl Backend for Spy {
    fn list(&self, q: &Values) -> Result<Vec<Record>, BackendError> {
        self.lists.lock().unwrap().push(std::thread::current().id());
        self.inner.list(q)
    }
    fn supported(&self, c: &str) -> bool {
        self.inner.supported(c)
    }
    fn change(&self, r: &str, e: &fs_types::Edit) -> Result<Outcome, BackendError> {
        self.inner.change(r, e)
    }
    fn run(&self, n: &str, r: Option<&str>, i: &Values) -> Result<Outcome, BackendError> {
        self.inner.run(n, r, i)
    }
}

/// Create a record with `Grace` while a search is set; the work runs on another thread, the answer is put in here.
/// Returns how often the program was asked on this thread while the answer went in.
fn asked_here_while_finishing(reject_search: bool) -> usize {
    let mem = Memory::new(vec![
        record("urn:1", &[("name", &["Ada"])]),
        record("urn:2", &[("name", &["Alan"])]),
    ]);
    let spy = Arc::new(Spy {
        inner: mem.clone(),
        lists: Mutex::new(Vec::new()),
    });
    let mut s = Session::new(contact(), spy.clone(), clock());
    assert!(s.begin("find"));
    s.form_mut().unwrap().set("term", 0, "Ada");
    assert!(s.submit());
    assert!(s.search_active());
    *mem.reject_search.lock().unwrap() = reject_search;
    assert!(s.begin("create"));
    s.form_mut().unwrap().set("name", 0, "Grace");
    let job = s.submit_begin().unwrap();
    let outcome = std::thread::spawn(move || job.run()).join().unwrap();
    let here = std::thread::current().id();
    let before = spy
        .lists
        .lock()
        .unwrap()
        .iter()
        .filter(|t| **t == here)
        .count();
    assert!(s.submit_finish(outcome));
    let after = spy
        .lists
        .lock()
        .unwrap()
        .iter()
        .filter(|t| **t == here)
        .count();
    after - before
}

#[test]
fn the_program_is_not_asked_under_the_lock_when_the_new_record_is_not_in_the_narrowed_list() {
    assert_eq!(asked_here_while_finishing(false), 0);
}

#[test]
fn the_program_is_not_asked_under_the_lock_when_the_narrowed_search_is_refused_now() {
    assert_eq!(asked_here_while_finishing(true), 0);
}

#[test]
fn text_typed_after_sending_is_told_not_dropped_silently() {
    let mem = Memory::new(vec![]);
    let mut s = Session::new(contact(), mem.clone(), clock());
    assert!(s.begin("create"));
    s.form_mut().unwrap().set("name", 0, "Grace");
    let job = s.submit_begin().unwrap();
    s.form_mut().unwrap().set("name", 0, "Grace!"); // typed while the command runs
    assert!(s.submit_finish(job.run()));
    assert_eq!(mem.records.lock().unwrap()[0].values["name"], ["Grace"]);
    assert!(
        s.messages()
            .current()
            .iter()
            .any(|m| m.code == "edited-after-send"),
        "the person is told that the late text was not saved"
    );
}

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
fn a_write_without_an_answer_says_the_result_is_unknown_and_frees_the_session() {
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
            .any(|m| m.code == "unknown-result")
    );
}
