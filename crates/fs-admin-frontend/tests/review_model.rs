//! Tests for the findings of the independent review (2026-11-10): model level.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod common;

use std::sync::Arc;

use common::{clock, contact, values};
use fs_admin_frontend::{FormModel, MessageKind, Messages, plan_commands};
use fs_types::{CommandKind, RecordProblem, TypeProblem};

// ---- findings of the independent review (2026-11-10) ----

#[test]
fn review_4_a_change_or_delete_without_a_reference_to_the_record_is_not_offered() {
    let d = contact().command(fs_types::CommandDescription::new(
        "rename",
        CommandKind::Change,
        "command.rename",
    ));
    let d = d.command(fs_types::CommandDescription::new(
        "drop",
        CommandKind::Delete,
        "command.drop",
    ));
    let names: Vec<String> = plan_commands(&d).into_iter().map(|p| p.name).collect();
    assert!(
        !names.contains(&"rename".to_owned()) && !names.contains(&"drop".to_owned()),
        "{names:?}"
    );
    let hidden = fs_admin_frontend::unoffered_commands(&d);
    assert_eq!(
        hidden,
        ["rename", "drop"],
        "and the frontend says which, it is not silent"
    );
}

#[test]
fn review_6_submit_returns_only_what_changed_and_a_cleared_field_as_empty() {
    let fields = contact().find_command("change").unwrap().input.clone();
    let initial = values(&[
        ("id", &["urn:1"]),
        ("name", &["Ada"]),
        ("emails", &["a@b.test", "c@d.test"]),
    ]);
    let fields: Vec<_> = fields.into_iter().filter(|f| f.name != "id").collect();
    let mut form = FormModel::new(&fields, &initial);
    let first = form.submit().unwrap();
    assert_eq!(
        first.keys().collect::<Vec<_>>(),
        ["name"],
        "nothing changed: only the required field goes"
    );
    form.set("name", 0, "Ada King");
    form.set("emails", 0, "");
    form.remove_value("emails", 0);
    form.set("emails", 0, "");
    let sent = form.submit().unwrap();
    assert_eq!(sent["name"], ["Ada King"]);
    assert_eq!(sent["emails"], Vec::<String>::new(), "cleared");
}

#[test]
fn review_8_a_listener_may_ask_the_messages_without_deadlock() {
    let m = Arc::new(Messages::new(clock()));
    let inner = Arc::clone(&m);
    m.subscribe(move || {
        let _ = inner.current();
        inner.subscribe(|| {});
    });
    let m2 = Arc::clone(&m);
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        m2.enter("list");
        m2.raise(MessageKind::Error, "unavailable");
        tx.send(()).unwrap();
    });
    assert!(
        rx.recv_timeout(std::time::Duration::from_secs(5)).is_ok(),
        "deadlock"
    );
}

#[test]
fn review_11_removing_a_slot_moves_the_marks_of_the_slots_after_it() {
    let fields = contact().find_command("create").unwrap().input.clone();
    let mut form = FormModel::new(
        &fields,
        &values(&[
            ("name", &["x"]),
            ("emails", &["a@b.test", "bad", "c@d.test"]),
        ]),
    );
    form.set("emails", 1, "still bad");
    assert_eq!(form.visible_issues().len(), 1);
    form.refuse(vec![fs_types::RecordIssue {
        field: "emails".into(),
        index: 2,
        problem: RecordProblem::Value(TypeProblem::Empty),
    }]);
    form.remove_value("emails", 0);
    let shown = form.visible_issues();
    assert!(
        shown.iter().any(|i| i.index == 0),
        "the finding on 'still bad' moved to 0: {shown:?}"
    );
    assert!(
        shown
            .iter()
            .any(|i| i.index == 1 && i.problem == RecordProblem::Value(TypeProblem::Empty)),
        "the refusal of slot 2 moved to 1: {shown:?}"
    );
    form.set("emails", 0, "fixed@b.test");
    assert!(
        form.visible_issues()
            .iter()
            .any(|i| i.index == 1 && i.problem == RecordProblem::Value(TypeProblem::Empty)),
        "editing slot 0 does not clear the refusal of slot 1"
    );
}
