//! The form of an edit: one field per value of the draft.
//!
//! Pattern: Adapter over `blinc_cn`'s input and text area. Typing reports to the draft and does not
//! redraw. A choice is a row of buttons at fixed places: pressing one changes the draft and draws again.

use std::sync::Arc;

use blinc_cn::components::button::ButtonVariant;
use blinc_cn::components::input::{InputBuilder, InputSize, input};
use blinc_cn::components::textarea::{TextareaSize, textarea};
use blinc_layout::div::{Div, div};
use blinc_layout::text;

use super::actions::{Act, TaskActions, button};
use super::detail::with_marks;
use crate::field_look::paired;
use crate::input_slots::InputSlots;
use crate::tasks::draft::{DraftField, FieldKind, FieldOption};
use crate::tasks::edit::TaskEdit;

/// The form for `edit`, added to `page`.
pub(super) fn form(
    edit: &TaskEdit,
    name: &str,
    actions: &TaskActions,
    slots: &InputSlots,
    problems: &dyn Fn(&str) -> Option<String>,
    mut page: Div,
) -> Div {
    if !edit.draft.marks.is_empty() {
        page = page.child(with_marks(
            div().class("fs-tasks-head").flex_row().flex_wrap(),
            &edit.draft.marks,
        ));
    }
    if let Some(note) = &edit.draft.note {
        page = page.child(div().class("fs-tasks-note").child(text(note)));
    }
    if let Some(error) = &edit.error {
        page = page.child(div().class("fs-tasks-failed").child(text(error)));
    }
    for field in &edit.draft.fields {
        page = page.child(element(
            field,
            edit.id,
            name,
            actions,
            slots,
            problems(&field.key),
        ));
    }
    page
}

fn element(
    field: &DraftField,
    edit: u64,
    name: &str,
    actions: &TaskActions,
    slots: &InputSlots,
    problem: Option<String>,
) -> Div {
    let id = format!("{name}-field-{}", field.key);
    let holder = div().class("fs-tasks-input").w_full();
    let (typed, key) = (Arc::clone(&actions.typed), field.key.clone());
    match &field.kind {
        FieldKind::Choice(options) => holder
            .flex_col()
            .child(div().class("fs-tasks-label").child(text(&field.label)))
            .child(choices(field, options, &id, actions)),
        FieldKind::LongText => {
            let state = slots.area(edit, &field.key, &field.value);
            holder.id(id).child(
                textarea(&state)
                    .size(TextareaSize::Small)
                    .label(field.label.clone())
                    .w_full()
                    .rows(4)
                    .on_change(move |value| typed(&key, value)),
            )
        }
        _ => {
            let data = slots.line(edit, &field.key, &field.value);
            holder.id(id).child(with_problem(
                paired(input(&data))
                    .size(InputSize::Small)
                    .label(field.label.clone())
                    .w_full()
                    .on_change(move |value| typed(&key, value)),
                problem,
            ))
        }
    }
}

fn choices(field: &DraftField, options: &[FieldOption], id: &str, actions: &TaskActions) -> Div {
    let mut row = div().class("fs-tasks-choices").flex_row().flex_wrap();
    for option in options {
        let (choose, key, value) = (
            Arc::clone(&actions.choose),
            field.key.clone(),
            option.key.clone(),
        );
        let act: Act = Arc::new(move || choose(&key, &value));
        let variant = if field.value == option.key {
            ButtonVariant::Primary
        } else {
            ButtonVariant::Secondary
        };
        row = row.child(button(
            format!("{id}-{}", option.key),
            &option.label,
            variant,
            &act,
        ));
    }
    row
}

/// The field with what is wrong with it under it, drawn as an error of the field.
fn with_problem(field: InputBuilder, problem: Option<String>) -> InputBuilder {
    match problem {
        Some(message) => field.error(message),
        None => field,
    }
}
