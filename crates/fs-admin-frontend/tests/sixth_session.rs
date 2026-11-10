//! Tests of the locked control "coming soon": a command the core does not carry out yet is shown locked and unlocks
//! when the core reports it, without a change in the frontend.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod common;

use common::{Memory, clock, contact, record};
use fs_admin_frontend::{Placement, Screen, Session};

fn open() -> (Session, std::sync::Arc<Memory>) {
    let mem = Memory::new(vec![record("urn:1", &[("name", &["Ada"])])]);
    (Session::new(contact(), mem.clone(), clock()), mem)
}

#[test]
fn a_command_the_core_does_not_support_is_still_listed_but_cannot_be_started() {
    let (mut s, mem) = open();
    mem.unsupported.lock().unwrap().insert("create".into());
    assert!(s.reload()); // the core is asked when the records are read
    assert!(
        s.commands(Placement::OnList)
            .iter()
            .any(|p| p.name == "create"),
        "it stays visible"
    );
    assert!(!s.is_supported("create"));
    assert!(!s.begin("create"));
    assert_eq!(*s.screen(), Screen::List);
}

#[test]
fn the_command_unlocks_when_the_core_reports_it() {
    let (mut s, mem) = open();
    mem.unsupported.lock().unwrap().insert("create".into());
    s.reload();
    assert!(!s.begin("create"));
    mem.unsupported.lock().unwrap().clear();
    s.reload();
    assert!(s.is_supported("create"));
    assert!(s.begin("create"));
}

#[test]
fn a_record_command_is_locked_the_same_way() {
    let (mut s, mem) = open();
    mem.unsupported.lock().unwrap().insert("delete".into());
    s.reload();
    assert!(s.open_detail("urn:1"));
    assert!(!s.begin("delete"));
    assert!(s.begin("change"), "the others are not affected");
}

/// A program that implements only the two required methods and says nothing about support.
struct Silent;

impl fs_admin_frontend::Backend for Silent {
    fn list(
        &self,
        _query: &fs_admin_frontend::Values,
    ) -> Result<Vec<fs_admin_frontend::Record>, fs_admin_frontend::BackendError> {
        Ok(vec![record("urn:1", &[("name", &["Ada"])])])
    }

    fn run(
        &self,
        _name: &str,
        _input: &fs_admin_frontend::Values,
    ) -> Result<fs_admin_frontend::Outcome, fs_admin_frontend::BackendError> {
        Err(fs_admin_frontend::BackendError::NotFound)
    }
}

#[test]
fn a_program_that_does_not_answer_unlocks_nothing() {
    let mut s = Session::new(contact(), std::sync::Arc::new(Silent), clock());
    for p in s.commands(Placement::OnList) {
        assert!(
            !s.is_supported(&p.name),
            "{} was unlocked by silence",
            p.name
        );
    }
    assert!(!s.begin("create") && !s.begin("find"));
    assert!(s.open_detail("urn:1"));
    assert!(!s.begin("change") && !s.begin("delete"));
}
