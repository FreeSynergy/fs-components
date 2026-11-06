//! The form of an edit: one field per value of the draft.
//!
//! Pattern: Adapter over `blinc_cn`'s input and text area + Memento — [`InputSlots`] keeps one input
//! state per field for the whole edit, so a redraw (a failed save, a pressed flag) keeps text and
//! cursor. Typing reports to the draft and does not redraw. A flag is a plain element: pressing it
//! changes the draft and draws again.

use std::collections::HashMap;
use std::sync::{Arc, Mutex, PoisonError};

use blinc_cn::components::input::{InputSize, input};
use blinc_cn::components::textarea::{TextareaSize, textarea};
use blinc_layout::div::{Div, div};
use blinc_layout::text;
use blinc_layout::widgets::text_area::{SharedTextAreaState, text_area_state};
use blinc_layout::widgets::text_input::{SharedTextInputData, text_input_data};

use super::actions::CalendarActions;
use super::detail::with_marks;
use crate::calendar::draft::{DraftField, FieldKind};
use crate::calendar::edit::ItemEdit;
use crate::calendar::labels::CalendarLabels;

/// The input states of one edit.
#[derive(Default)]
struct Slots {
    edit: u64,
    lines: HashMap<String, SharedTextInputData>,
    areas: HashMap<String, SharedTextAreaState>,
}

/// The input states of the edit in progress, tied to that edit by its id.
#[derive(Clone, Default)]
pub(in crate::calendar) struct InputSlots(Arc<Mutex<Slots>>);

impl InputSlots {
    /// The state of a one-line `field`, holding `value`. The lock of the slots is let go before the
    /// input's own lock is taken.
    fn line(&self, edit: u64, field: &str, value: &str) -> SharedTextInputData {
        let data = {
            let mut slots = self.0.lock().unwrap_or_else(PoisonError::into_inner);
            slots.for_edit(edit);
            Arc::clone(
                slots
                    .lines
                    .entry(field.to_string())
                    .or_insert_with(text_input_data),
            )
        };
        {
            let mut held = data.lock().unwrap_or_else(PoisonError::into_inner);
            if held.value != value {
                held.value = value.to_string();
                held.cursor = value.chars().count();
                held.selection_start = None;
            }
        }
        data
    }

    /// The state of a many-line `field`; holds `value` from when it is made, and after that what
    /// was typed into it.
    fn area(&self, edit: u64, field: &str, value: &str) -> SharedTextAreaState {
        let mut slots = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        slots.for_edit(edit);
        Arc::clone(slots.areas.entry(field.to_string()).or_insert_with(|| {
            let state = text_area_state();
            state
                .lock()
                .unwrap_or_else(PoisonError::into_inner)
                .set_value(value);
            state
        }))
    }
}

impl Slots {
    fn for_edit(&mut self, edit: u64) {
        if self.edit != edit {
            *self = Self {
                edit,
                ..Self::default()
            };
        }
    }
}

/// The form for `edit`, added to `page`.
pub(super) fn form(
    edit: &ItemEdit,
    labels: &CalendarLabels,
    name: &str,
    actions: &CalendarActions,
    slots: &InputSlots,
    mut page: Div,
) -> Div {
    if !edit.draft.marks.is_empty() {
        page = page.child(with_marks(
            div().class("fs-calendar-title").flex_row(),
            &edit.draft.marks,
        ));
    }
    if let Some(note) = &edit.draft.note {
        page = page.child(div().class("fs-calendar-note").child(text(note)));
    }
    if let Some(error) = &edit.error {
        page = page.child(div().class("fs-calendar-failed").child(text(error)));
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
    if field.read_only {
        return holder
            .id(id)
            .flex_row()
            .class("fs-calendar-field")
            .child(div().class("fs-calendar-label").child(text(&field.label)))
            .child(
                div()
                    .class("fs-calendar-value")
                    .flex_1()
                    .child(text(&field.value)),
            );
    }
    let (typed, key) = (Arc::clone(&actions.typed), field.key.clone());
    match field.kind {
        FieldKind::Flag => {
            let (toggle, key) = (Arc::clone(&actions.toggle), field.key.clone());
            let word = if field.value == "true" {
                &labels.yes
            } else {
                &labels.no
            };
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
                input(&data)
                    .size(InputSize::Small)
                    .label(field.label.clone())
                    .w_full()
                    .on_change(move |value| typed(&key, value)),
            )
        }
    }
}
