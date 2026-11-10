//! Tests for the findings of the independent review (2026-11-10): model level.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod common;

use std::sync::Arc;

use common::{clock, contact, values};
use fs_admin_frontend::{FormModel, MessageKind, Messages, Placement, plan_commands};
use fs_types::{RecordProblem, TypeProblem};

// ---- findings of the independent review (2026-11-10) ----

#[test]
fn review_4_change_and_delete_need_no_reference_the_id_travels_in_the_call() {
    let plans = plan_commands(&contact());
    let by = |n: &str| plans.iter().find(|p| p.name == n).unwrap();
    assert_eq!(by("change").placement, Placement::OnRecord);
    assert_eq!(by("delete").placement, Placement::OnRecord);
    assert!(by("change").bound.is_none() && by("delete").bound.is_none());
    assert!(
        by("delete").form.is_empty(),
        "delete asks, it types nothing"
    );
    let form: Vec<&str> = by("change").form.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(
        form,
        ["name", "emails", "phone", "note"],
        "change edits the fields of the type"
    );
}

#[test]
fn review_6_a_change_is_an_edit_of_only_what_changed_and_a_cleared_field_has_no_values() {
    let fields = contact().fields;
    let initial = values(&[("name", &["Ada"]), ("emails", &["a@b.test", "c@d.test"])]);
    let mut form = FormModel::new(&fields, &initial);
    assert!(
        form.submit_changes().unwrap().fields.is_empty(),
        "nothing changed, nothing in the edit"
    );
    form.set("name", 0, "Ada King");
    form.set("emails", 0, "");
    form.remove_value("emails", 0);
    form.set("emails", 0, "");
    let edit = form.submit_changes().unwrap();
    let get = |n: &str| edit.fields.iter().find(|f| f.path == [n]).unwrap();
    assert_eq!(get("name").values, ["Ada King"]);
    assert!(get("emails").values.is_empty(), "cleared");
    assert_eq!(edit.fields.len(), 2, "phone and note were not touched");
    assert_eq!(
        get("name").basic,
        fs_types::BasicType::Text,
        "typed, no JSON"
    );
}

#[test]
fn review_6_a_create_form_still_sends_values_of_what_was_typed() {
    let fields = contact().find_command("create").unwrap().input.clone();
    let mut form = FormModel::new(&fields, &values(&[]));
    form.set("name", 0, "Ada");
    let sent = form.submit().unwrap();
    assert_eq!(sent["name"], ["Ada"]);
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

#[test]
fn third_3_a_refusal_on_a_slot_that_is_not_there_points_at_the_last_slot_and_does_not_block_sending()
 {
    let fields = contact().find_command("create").unwrap().input.clone();
    let mut form = FormModel::new(
        &fields,
        &values(&[("name", &["x"]), ("emails", &["a@b.test"])]),
    );
    let issue = |index| fs_types::RecordIssue {
        field: "emails".into(),
        index,
        problem: RecordProblem::Value(TypeProblem::Character),
    };
    form.refuse(vec![issue(7), issue(7)]);
    let shown = form.visible_issues();
    assert_eq!(shown.len(), 1, "the same finding is shown once");
    assert_eq!(shown[0].index, 0);
    assert!(
        form.submit().is_ok(),
        "the program's refusal does not stop a new try"
    );
}
