//! Tests of the pure model against the traits: form, list, commands, messages.
//!
//! Pattern: none (tests).
#![allow(clippy::pedantic)]

#[path = "support/mod.rs"]
mod common;

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use common::{clock, contact, record, values};
use fs_admin_frontend::{
    Direction, FormModel, ListModel, MessageKind, Messages, Placement, plan_commands,
};
use fs_types::{CommandKind, RecordProblem, TypeProblem};

fn people() -> Vec<fs_admin_frontend::Record> {
    vec![
        record(
            "urn:1",
            &[
                ("name", &["Ada Lovelace"]),
                ("emails", &["ada@example.org", "ada@analytical.test"]),
            ],
        ),
        record(
            "urn:2",
            &[
                ("name", &["Charles Babbage"]),
                ("phone", &["+442071234567"]),
            ],
        ),
        record("urn:3", &[("name", &["Alan Turing"])]),
        record("urn:4", &[("emails", &["anon@example.org"])]),
    ]
}

#[test]
fn columns_leave_out_what_does_not_fit_a_cell() {
    let list = ListModel::new(&contact(), people());
    let names: Vec<&str> = list
        .columns()
        .iter()
        .map(|c| c.field.name.as_str())
        .collect();
    assert_eq!(
        names,
        ["name", "emails", "phone"],
        "note is multi-line text and has no column"
    );
}

#[test]
fn many_fields_keep_the_required_ones_in_description_order() {
    let mut d = contact();
    for i in 0..8 {
        d = d.field(fs_types::FieldDescription::new(
            &format!("extra{i}"),
            fs_types::BasicType::Text,
        ));
    }
    let list = ListModel::new(&d, vec![]);
    let names: Vec<&str> = list
        .columns()
        .iter()
        .map(|c| c.field.name.as_str())
        .collect();
    assert_eq!(names.len(), 6);
    assert_eq!(names[0], "name", "required stays");
    assert_eq!(names[1], "emails");
}

#[test]
fn sort_a_to_z_then_z_to_a_then_back() {
    let mut list = ListModel::new(&contact(), people());
    let order = |l: &ListModel| {
        l.rows()
            .iter()
            .map(|r| r.id.as_str().to_owned())
            .collect::<Vec<_>>()
    };
    list.cycle_sort(0);
    assert_eq!(list.sort(), Some((0, Direction::Ascending)));
    assert_eq!(order(&list), ["urn:1", "urn:3", "urn:2", "urn:4"]);
    list.cycle_sort(0);
    assert_eq!(
        order(&list),
        ["urn:2", "urn:3", "urn:1", "urn:4"],
        "descending, the empty one still last"
    );
    list.cycle_sort(0);
    assert_eq!(list.sort(), None);
    assert_eq!(order(&list), ["urn:1", "urn:2", "urn:3", "urn:4"]);
    assert!(!list.cycle_sort(9));
}

#[test]
fn filter_looks_in_every_value_without_case() {
    let mut list = ListModel::new(&contact(), people());
    list.set_filter("  EXAMPLE.org ");
    assert_eq!(list.rows().len(), 2);
    list.set_filter("");
    assert_eq!(list.rows().len(), 4);
}

#[test]
fn selection_survives_new_records_only_while_the_record_is_there() {
    let mut list = ListModel::new(&contact(), people());
    assert!(!list.select("urn:99"));
    assert!(list.select("urn:2"));
    list.set_records(people());
    assert_eq!(list.selected(), Some("urn:2"));
    list.set_records(vec![record("urn:1", &[])]);
    assert_eq!(list.selected(), None);
}

#[test]
fn plans_put_record_commands_with_the_record() {
    let plans = plan_commands(&contact());
    let by = |n: &str| plans.iter().find(|p| p.name == n).unwrap();
    assert_eq!(by("create").placement, Placement::OnList);
    assert_eq!(by("find").placement, Placement::OnList);
    assert_eq!(by("domains").placement, Placement::OnList);
    assert_eq!(by("change").placement, Placement::OnRecord);
    assert_eq!(by("change").bound.as_deref(), Some("id"));
    let form: Vec<&str> = by("change").form.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(form, ["name", "emails"], "the bound id is not typed");
    assert!(by("delete").confirm && by("delete").kind == CommandKind::Delete);
    assert!(!by("change").confirm);
}

#[test]
fn form_shows_a_missing_value_only_after_a_try_to_send() {
    let plans = plan_commands(&contact());
    let mut form = FormModel::new(&plans[0].form, &values(&[]));
    assert!(form.visible_issues().is_empty());
    assert_eq!(form.issues().len(), 1, "name is required");
    let found = form.submit().unwrap_err();
    assert_eq!(found[0].problem, RecordProblem::Missing);
    assert_eq!(form.visible_issues().len(), 1);
}

#[test]
fn form_marks_a_bad_value_where_the_person_typed() {
    let plans = plan_commands(&contact());
    let mut form = FormModel::new(&plans[0].form, &values(&[]));
    assert!(form.set("emails", 0, "not an address"));
    let seen = form.visible_issues();
    assert_eq!(seen.len(), 1);
    assert_eq!((seen[0].field.as_str(), seen[0].index), ("emails", 0));
    assert!(matches!(seen[0].problem, RecordProblem::Value(_)));
    assert!(!form.set("nope", 0, "x"));
}

#[test]
fn form_submits_canonical_values_without_empty_slots() {
    let plans = plan_commands(&contact());
    let mut form = FormModel::new(&plans[0].form, &values(&[]));
    form.set("name", 0, "  Ada  ");
    assert!(form.add_value("emails"));
    assert!(form.add_value("emails"));
    form.set("emails", 0, "Ada@Example.ORG");
    form.set("emails", 2, "ada@other.test");
    let sent = form.submit().unwrap();
    assert_eq!(sent["name"], ["Ada"]);
    assert_eq!(
        sent["emails"],
        ["Ada@example.org", "ada@other.test"],
        "domain lower case, empty slot dropped"
    );
}

#[test]
fn form_single_fields_take_one_slot_and_the_last_slot_is_emptied_not_removed() {
    let plans = plan_commands(&contact());
    let mut form = FormModel::new(&plans[0].form, &values(&[("name", &["a", "b"])]));
    assert_eq!(form.values("name").len(), 1);
    assert!(!form.add_value("name"));
    form.set("name", 0, "Ada");
    assert!(form.remove_value("name", 0));
    assert_eq!(form.values("name"), [""]);
    assert!(!form.remove_value("name", 3));
}

#[test]
fn form_knows_when_it_is_dirty_and_clears_a_refusal_on_edit() {
    let plans = plan_commands(&contact());
    let initial = values(&[("name", &["Ada"])]);
    let mut form = FormModel::new(&plans[0].form, &initial);
    assert!(!form.is_dirty());
    form.set("name", 0, "Ada L");
    assert!(form.is_dirty());
    form.refuse(vec![fs_types::RecordIssue {
        field: "name".into(),
        index: 0,
        problem: RecordProblem::Value(TypeProblem::Empty),
    }]);
    assert_eq!(
        form.visible_issues().len(),
        1,
        "a refusal by the program shows at once"
    );
    form.set("name", 0, "Ada Lovelace");
    assert!(form.visible_issues().is_empty());
}

#[test]
fn success_goes_by_itself_error_stays_and_neither_follows_to_another_screen() {
    let clock = clock();
    let m = Messages::new(clock.clone());
    let seen = Arc::new(AtomicUsize::new(0));
    let s = Arc::clone(&seen);
    m.subscribe(move || {
        s.fetch_add(1, Ordering::SeqCst);
    });
    m.enter("list");
    let ok = m.raise(MessageKind::Success, "done");
    let bad = m.raise(MessageKind::Error, "unavailable");
    assert_eq!(m.current().len(), 2);
    clock.advance(3999);
    assert_eq!(m.current().len(), 2);
    clock.advance(1);
    let now = m.current();
    assert_eq!(now.len(), 1, "success expired");
    assert_eq!(now[0].id, bad);
    clock.advance(1_000_000);
    assert_eq!(m.current().len(), 1, "an error never expires");
    m.enter("detail:1");
    assert!(
        m.current().is_empty(),
        "another screen shows nothing of the old one"
    );
    assert!(!m.dismiss(ok));
    assert!(seen.load(Ordering::SeqCst) >= 4, "listeners are told");
}

#[test]
fn solve_closes_the_errors_with_that_code_and_dismiss_one() {
    let m = Messages::new(clock());
    m.enter("list");
    m.raise(MessageKind::Error, "unavailable");
    let other = m.raise(MessageKind::Error, "not-found");
    m.solve("unavailable");
    assert_eq!(m.current().len(), 1);
    assert!(m.dismiss(other));
    assert!(m.current().is_empty());
}
