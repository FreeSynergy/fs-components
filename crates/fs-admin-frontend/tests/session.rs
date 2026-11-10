//! Tests of the session: the transitions of the screens and what the commands do, over a `Backend` in memory.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod common;

use common::{Memory, clock, contact, record};
use fs_admin_frontend::{MessageKind, Placement, Screen, Session};

fn open() -> (
    Session,
    std::sync::Arc<Memory>,
    std::sync::Arc<fs_admin_frontend::ManualClock>,
) {
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
    let c = clock();
    (Session::new(contact(), mem.clone(), c.clone()), mem, c)
}

fn type_in(s: &mut Session, field: &str, i: usize, text: &str) {
    assert!(s.form_mut().unwrap().set(field, i, text));
}

#[test]
fn opens_on_the_list_with_the_records() {
    let (s, _, _) = open();
    assert_eq!(*s.screen(), Screen::List);
    assert_eq!(s.list().rows().len(), 2);
    assert!(s.messages().current().is_empty());
}

#[test]
fn a_program_that_is_down_gives_an_error_that_stays_and_goes_when_it_is_back() {
    let mem = Memory::new(vec![record("urn:1", &[("name", &["Ada"])])]);
    *mem.down.lock().unwrap() = true;
    let c = clock();
    let mut s = Session::new(contact(), mem.clone(), c.clone());
    assert!(s.list().rows().is_empty());
    let m = s.messages().current();
    assert_eq!(
        (m[0].kind, m[0].code.as_str()),
        (MessageKind::Error, "unavailable")
    );
    c.advance(60_000);
    assert_eq!(
        s.messages().current().len(),
        1,
        "never silent, never by itself"
    );
    *mem.down.lock().unwrap() = false;
    s.reload();
    assert!(s.messages().current().is_empty());
    assert_eq!(s.list().rows().len(), 1);
}

#[test]
fn create_checks_first_then_the_program_then_shows_the_new_record() {
    let (mut s, mem, _) = open();
    assert!(s.begin("create"));
    assert!(matches!(s.screen(), Screen::Command { record: None, .. }));
    assert!(!s.submit(), "name is missing, nothing was sent");
    assert_eq!(mem.records.lock().unwrap().len(), 2);
    assert_eq!(s.form().unwrap().visible_issues().len(), 1);
    type_in(&mut s, "name", 0, "Grace Hopper");
    type_in(&mut s, "emails", 0, "grace@navy.test");
    assert!(s.submit());
    assert_eq!(mem.records.lock().unwrap().len(), 3);
    let Screen::Detail(id) = s.screen().clone() else {
        panic!("{:?}", s.screen())
    };
    assert_eq!(s.list().selected(), Some(id.as_str()));
    let name = s
        .detail()
        .into_iter()
        .find(|e| e.field.name == "name")
        .unwrap();
    assert_eq!(name.values, ["Grace Hopper"]);
    let ok = s.messages().current();
    assert_eq!(
        (ok[0].kind, ok[0].code.as_str()),
        (MessageKind::Success, "done")
    );
    assert!(s.form().is_none());
}

#[test]
fn the_success_goes_when_the_screen_changes() {
    let (mut s, _, _) = open();
    s.begin("create");
    type_in(&mut s, "name", 0, "Grace");
    assert!(s.submit());
    assert_eq!(s.messages().current().len(), 1);
    s.open_list();
    assert!(s.messages().current().is_empty());
}

#[test]
fn change_starts_from_the_record_and_sends_the_record_id() {
    let (mut s, mem, _) = open();
    assert!(!s.begin("change"), "no record selected");
    assert!(s.open_detail("urn:1"));
    assert!(s.begin("change"));
    let form = s.form().unwrap();
    assert_eq!(form.values("name"), ["Ada Lovelace"]);
    assert_eq!(form.values("emails"), ["ada@example.org"]);
    assert!(!form.is_dirty());
    type_in(&mut s, "name", 0, "Ada King");
    assert!(s.submit());
    let stored = mem.records.lock().unwrap();
    assert_eq!(stored[0].values["name"], ["Ada King"]);
    assert_eq!(stored[0].id, "urn:1");
    drop(stored);
    assert_eq!(*s.screen(), Screen::Detail("urn:1".into()));
}

#[test]
fn delete_asks_first_and_cancel_goes_back_to_the_record() {
    let (mut s, mem, _) = open();
    s.open_detail("urn:2");
    assert!(s.begin("delete"));
    assert!(matches!(s.screen(), Screen::Confirm { .. }));
    s.cancel();
    assert_eq!(*s.screen(), Screen::Detail("urn:2".into()));
    assert_eq!(mem.records.lock().unwrap().len(), 2);
    s.begin("delete");
    assert!(s.submit());
    assert_eq!(mem.records.lock().unwrap().len(), 1);
    assert_eq!(*s.screen(), Screen::List);
    assert_eq!(s.list().selected(), None);
}

#[test]
fn a_record_that_is_gone_is_an_error_not_a_crash() {
    let (mut s, mem, _) = open();
    s.open_detail("urn:2");
    s.begin("delete");
    mem.records.lock().unwrap().retain(|r| r.id != "urn:2");
    assert!(!s.submit());
    let m = s.messages().current();
    assert_eq!(m[0].code, "not-found");
}

#[test]
fn the_program_may_refuse_what_the_form_let_through_and_the_form_marks_it() {
    let (mut s, mem, _) = open();
    *mem.forbidden.lock().unwrap() = Some("Mallory".into());
    s.begin("create");
    type_in(&mut s, "name", 0, "Mallory");
    assert!(!s.submit());
    assert!(
        matches!(s.screen(), Screen::Command { .. }),
        "the form stays, with what was typed"
    );
    assert_eq!(s.form().unwrap().values("name"), ["Mallory"]);
    let shown = s.form().unwrap().visible_issues();
    assert_eq!(shown.len(), 1);
    assert_eq!(shown[0].field, "name");
    assert_eq!(mem.records.lock().unwrap().len(), 2);
    type_in(&mut s, "name", 0, "Grace");
    assert!(s.submit());
}

#[test]
fn find_narrows_the_list_and_open_list_widens_it() {
    let (mut s, _, _) = open();
    assert!(s.begin("find"));
    type_in(&mut s, "text", 0, "babbage");
    assert!(s.submit());
    assert_eq!(*s.screen(), Screen::List);
    assert_eq!(s.list().rows().len(), 1);
    s.open_list();
    assert_eq!(s.list().rows().len(), 2);
}

#[test]
fn a_command_with_a_result_shows_it() {
    let (mut s, _, _) = open();
    assert!(s.begin("domains"));
    assert!(s.submit());
    assert_eq!(*s.screen(), Screen::Result("domains".into()));
    assert_eq!(s.result().unwrap().values["domains"], ["example.org"]);
}

#[test]
fn unknown_things_change_nothing() {
    let (mut s, _, _) = open();
    assert!(!s.begin("nope"));
    assert!(!s.open_detail("urn:99"));
    assert!(!s.submit(), "nothing is open");
    assert_eq!(*s.screen(), Screen::List);
    assert_eq!(s.commands(Placement::OnList).len(), 3);
    assert_eq!(s.commands(Placement::OnRecord).len(), 2);
}
