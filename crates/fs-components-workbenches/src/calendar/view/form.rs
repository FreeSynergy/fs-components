//! The form of an edit: one field per value of the draft.
//!
//! Pattern: Adapter over `blinc_cn`'s input and text area. [`InputSlots`] keeps one input state per
//! field for the whole edit. Typing reports to the draft and does not redraw. A flag is a plain
//! element: pressing it changes the draft and draws again.

use std::sync::Arc;

use blinc_cn::components::input::{InputSize, input};
use blinc_cn::components::textarea::{TextareaSize, textarea};
use blinc_layout::div::{Div, div};
use blinc_layout::text;

use super::actions::CalendarActions;
use crate::calendar::draft::{DraftField, FieldKind};
use crate::calendar::edit::ItemEdit;
use crate::calendar::labels::CalendarLabels;
use crate::field_look::paired;
use crate::input_slots::InputSlots;

/// The form for `edit`, added to `page`.
pub(super) fn form(
    edit: &ItemEdit,
    labels: &CalendarLabels,
    name: &str,
    actions: &CalendarActions,
    slots: &InputSlots,
    mut page: Div,
) -> Div {
    if let Some(note) = &edit.draft.note {
        page = page.child(div().class("fs-calendar-note").child(text(note)));
    }
    for field in &edit.draft.fields {
        page = page.child(element(field, edit.id, labels, name, actions, slots));
    }
    page
}

fn element(
    field: &DraftField,
    edit: u64,
    labels: &CalendarLabels,
    name: &str,
    actions: &CalendarActions,
    slots: &InputSlots,
) -> Div {
    let id = format!("{name}-field-{}", field.key);
    let holder = div().class("fs-calendar-input").w_full();
    let word = if field.value == "true" {
        &labels.yes
    } else {
        &labels.no
    };
    if field.read_only {
        let shown = if field.kind == FieldKind::Flag {
            word
        } else {
            &field.value
        };
        return holder
            .id(id)
            .flex_row()
            .class("fs-calendar-field")
            .child(div().class("fs-calendar-label").child(text(&field.label)))
            .child(div().class("fs-calendar-value").flex_1().child(text(shown)));
    }
    let (typed, key) = (Arc::clone(&actions.typed), field.key.clone());
    match field.kind {
        FieldKind::Flag => {
            let (toggle, key) = (Arc::clone(&actions.toggle), field.key.clone());
            holder
                .flex_row()
                .class("fs-calendar-field")
                .child(div().class("fs-calendar-label").child(text(&field.label)))
                .child(
                    div()
                        .id(id)
                        .class("fs-calendar-flag")
                        .class(if field.value == "true" {
                            "fs-calendar-flag-on"
                        } else {
                            "fs-calendar-flag-off"
                        })
                        .on_click(move |_| toggle(&key))
                        .child(text(word).no_cursor()),
                )
        }
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
            holder.id(id).child(
                paired(input(&data))
                    .size(InputSize::Small)
                    .label(field.label.clone())
                    .w_full()
                    .on_change(move |value| typed(&key, value)),
            )
        }
    }
}
